import time
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.common.action_chains import ActionChains
from bs4 import BeautifulSoup

def get_current_price():
    driver = webdriver.Firefox()
    driver.set_page_load_timeout(30) # wait for max 30 seconds for the page to load
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
            price = float(e.get_text().replace(",","."))

            driver.quit()
            break
        except Excetion as e:
            if e == TimeoutException:
                print("Timeout on server, skipping this hour")
            if retries < 10:
                print("[{}] Error while loading page, refreshing and retrying...".format(retries))
                retries += 1
                driver.refresh()
                time.sleep(2)
            else:
                price = -1.0
                break

    return price
