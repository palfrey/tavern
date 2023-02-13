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

    def wait_for_pub():
        pubs = browser.find_elements(By.CLASS_NAME, "pubItem")
        pubNames = []
        for pub in pubs:
            currentPub = pub.find_element(By.CLASS_NAME, "pubName").text
            if pub.find_element(By.CLASS_NAME, "pubName").text == pubName:
                return True
            pubNames.append(currentPub)
        browser.log(f"Pubs: {pubNames}")
        return False

    browser.wait_until(wait_for_pub)
