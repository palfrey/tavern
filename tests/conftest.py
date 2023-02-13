import re
import time
from datetime import datetime
from os import environ
from typing import Any, Callable, List, Tuple, TypeVar, Union

import pytest
from retry import retry
from selenium import webdriver
from selenium.common.exceptions import (
    NoSuchElementException,
    StaleElementReferenceException,
    TimeoutException,
    WebDriverException,
)
from selenium.webdriver.chrome.options import Options as ChromeOptions
from selenium.webdriver.common.by import By
from selenium.webdriver.remote.webelement import WebElement
from selenium.webdriver.support.ui import WebDriverWait

T = TypeVar("T")


class Browser:
    DEFAULT_TIMEOUT = 10

    def __init__(self):
        options = ChromeOptions()
        options.accept_insecure_certs = True
        options.add_argument("--use-fake-device-for-media-stream")
        options.add_argument("--use-file-for-fake-video-capture=newfile.mjpeg")
        self.driver = webdriver.Remote(
            command_executor=environ["SELENIUM_URL"],
            options=options,
        )
        self.start = time.time()
        self.allowed: List[re.Pattern] = []

    def add_allowed_log_pattern(self, regex: re.Pattern):
        self.allowed.append(regex)

    def log(self, message):
        print(f"{(time.time()-self.start):.3f}: {message}")

    def goto(self, url):
        self.log(f"Going to {url}")
        self.driver.get(url)
        if self.check_logs():
            raise Exception

    def check_logs(self) -> bool:
        fail = False
        for entry in self.driver.get_log("browser"):
            if entry["level"] == "SEVERE":
                for check in self.allowed:
                    if check.search(entry["message"]) is not None:
                        break
                else:
                    self.log("Browser: %s" % entry)
                    fail = True
            else:
                self.log("Browser: %s" % entry)

        return fail

    def failure(self):
        when = datetime.now().isoformat(timespec="seconds").replace(":", "")
        self.driver.get_screenshot_as_file(f"screenshots/{when}.png")
        self.check_logs()

    def find_elements(
        self, by: By, selector: str, quiet: bool = False
    ) -> List[WebElement]:
        if not quiet:
            self.log(f"Finding {by} {selector}")
        res = self.driver.find_elements(by, selector)
        if self.check_logs():
            raise Exception
        return res

    def find_element(self, by: By, selector: str, quiet: bool = False) -> WebElement:
        if not quiet:
            self.log(f"Finding {by} {selector}")
        elements = self.find_elements(by, selector, quiet=True)
        if len(elements) == 1:
            return elements[0]
        elif len(elements) == 0:
            self.failure()
            raise NoSuchElementException
        else:
            raise WebDriverException(f"Expected 1 element, got {len(elements)}")

    def find_one_of(self, locators: List[Tuple[By, str]]) -> Union[WebElement, bool]:
        for locator in locators:
            try:
                element = self.find_element(*locator)
                if element is not None:
                    return element
            except NoSuchElementException:
                continue
        return False

    def wait_until(
        self, until_func: Callable[[], T], timeout: int = DEFAULT_TIMEOUT
    ) -> T:
        try:
            w = WebDriverWait(self.driver, timeout)
            return w.until(lambda driver: until_func()), timeout
        except TimeoutException:
            self.failure()
            raise

    def wait_until_not(
        self, until_func: Callable[[], T], timeout: int = DEFAULT_TIMEOUT
    ) -> T:
        try:
            w = WebDriverWait(self.driver, timeout)
            return w.until_not(lambda driver: until_func()), timeout
        except TimeoutException:
            self.failure()
            raise

    def wait_for_list(
        self, items: List[Tuple[By, str]], timeout: int = DEFAULT_TIMEOUT
    ) -> WebElement:
        return self.wait_until(lambda: self.find_one_of(items), timeout)

    def wait_for_elements(
        self, by: By, selector: str, timeout: int = DEFAULT_TIMEOUT
    ) -> List[WebElement]:
        return self.wait_until(lambda: self.find_elements(by, selector), timeout)

    def wait_for_missing(
        self, by: By, selector: str, timeout: int = DEFAULT_TIMEOUT
    ) -> None:
        return self.wait_until_not(lambda: self.find_element(by, selector), timeout)

    def wait_for_text(
        self, by: By, selector: str, expected: str, timeout: int = DEFAULT_TIMEOUT
    ) -> None:
        def _wait_for_text():
            current_text = self.get_text(by, selector)
            self.log(
                f"Found '{current_text}', expected '{expected}' for {by}, {selector}"
            )
            return current_text == expected

        return self.wait_until(_wait_for_text, timeout)

    def has_element(self, by: By, selector: str) -> bool:
        return self.find_element(by, selector) is not None

    @retry(StaleElementReferenceException)
    def click(self, by: By, selector: str) -> None:
        self.log(f"Clicking {by}, {selector}")
        element = self.find_element(by, selector, quiet=True)
        element.click()

    def enter_text(self, by: By, selector: str, text: str) -> None:
        self.log(f"Entering '{text}' on {by}, {selector}")
        element = self.find_element(by, selector, quiet=True)
        element.clear()
        element.send_keys(text)

    @retry(StaleElementReferenceException)
    def get_text(self, by: By, selector: str) -> str:
        element = self.find_element(by, selector)
        return element.get_property("value")

    def run_js(self, script: str) -> Any:
        return self.driver.execute_script(script)


@pytest.fixture
def browser():
    b = Browser()
    yield b
    b.driver.quit()
