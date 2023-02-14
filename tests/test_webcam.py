import uuid
from re import compile
from typing import Callable

from selenium.webdriver.common.by import By

from .conftest import Browser


def set_vite_allowed(browser: Browser):
    browser.add_allowed_log_pattern(compile("/@vite/client"))


def test_webcam(browser_factory: Callable[[], Browser]):
    browser = browser_factory()
    set_vite_allowed(browser)
    browser.goto("https://nginx:8000/")

    pubName = f"pub-{uuid.uuid4()}"
    browser.enter_text(By.ID, "pubName", pubName)
    browser.click(By.ID, "createPub")
    pubelement = browser.wait_for_element(By.ID, "currentPubName")
    assert pubelement.text == pubName

    tableName = f"table-{uuid.uuid4()}"
    browser.enter_text(By.ID, "tableName", tableName)
    browser.click(By.ID, "createTable")

    browser.wait_for_element(By.TAG_NAME, "video")
    # browser.screenshot()

    new_browser = browser_factory()
    set_vite_allowed(new_browser)
    new_browser.goto("https://nginx:8000/")
    new_browser.wait_for_element(By.ID, f"join-{pubName}").click()
    new_browser.wait_for_element(By.ID, f"join-{tableName}").click()

    videos = new_browser.wait_for_elements(By.TAG_NAME, "video")
    assert len(videos) == 2
    new_browser.screenshot()
