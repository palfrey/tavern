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

    video = browser.wait_for_element(By.TAG_NAME, "video")
    assert video.get_property("srcObject")["active"]
    # browser.screenshot()

    new_browser = browser_factory()
    set_vite_allowed(new_browser)
    new_browser.goto("https://nginx:8000/")
    new_browser.click(By.ID, f"join-{pubName}")
    new_browser.click(By.ID, f"join-{tableName}")

    def wait_videos():
        videos = new_browser.wait_for_elements(By.TAG_NAME, "video")
        browser.check_logs()
        assert len(videos) == 2
        for video in videos:
            srcObject = video.get_property("srcObject")
            if srcObject is None:
                return False
            assert srcObject["active"]
        return True

    new_browser.wait_until(wait_videos)
    new_browser.screenshot()
