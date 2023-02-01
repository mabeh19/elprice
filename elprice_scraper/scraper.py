import time
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.common.action_chains import ActionChains
from bs4 import BeautifulSoup

def get_current_price():
    driver = webdriver.Firefox()
    price = 0.0

    driver.get("https://www.energifyn.dk/kundeservice/kundeservice-el/faq-el/hvad-er-prisen-pa-el/")

    element = driver.find_element(By.ID, "declineButton")

    action = ActionChains(driver)
    action.click(element)
    action.perform()

    time.sleep(2)
    price = 0.0
    retries = 0

    while True:
        try:
            soup = BeautifulSoup(driver.page_source, features="html.parser")
            e = soup.find(class_ = "details-header__total")
            price = float(e.get_text())

            driver.quit()
            break
        except:
            if retries < 10:
                print("[{}] Error while loading page, refreshing and retrying...".format(retries))
                driver.refresh()
                time.sleep(2)
            else:
                break

    return price
