import uuid
from re import compile

from selenium.webdriver.common.by import By

from .conftest import Browser


def test_webcam(browser: Browser):
    browser.add_allowed_log_pattern(compile("/@vite/client"))
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
    browser.screenshot()
