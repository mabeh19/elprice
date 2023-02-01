import time
import socket
import scraper

ONE_MINUTE = 60
ONE_HOUR = 60 * 60
IP_ADDRESS = "10.0.2.15"
PORT = 35000

def upload_to_server(price):
    print("Creating socket...")
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    print("Connecting to {}:{}".format(IP_ADDRESS, PORT))
    s.connect((IP_ADDRESS, PORT))
    s.send("[[current price]] {}".format(price).encode('utf-8'))
    print("Data sent, closing connection...")
    s.close()

while True:
    try: 
        current_price = scraper.get_current_price()
     
        upload_to_server(current_price)

        print("Success! Going to sleep...")

        time.sleep(ONE_HOUR)
    except:
        print("Error occurred retrying in a minute")
        time.sleep(ONE_MINUTE)
