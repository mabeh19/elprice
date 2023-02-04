import time
import datetime
import socket
import scraper

ONE_MINUTE = 60
ONE_HOUR = 60 * 60
NEXT_HOUR = 1
IP_ADDRESS = "10.0.2.15"
PORT = 35000

def sleepUntil(hour):
    t = datetime.datetime.today()
    future = datetime.datetime(t.year, t.month, t.day, t.hour + hour, 0)
    if t.timestamp() > future.timestamp():
        future += datetime.timedelta(days=1)
    time.sleep((future-t).total_seconds())

def upload_to_server(price):
    print("Creating socket...")
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    print("Connecting to {}:{}".format(IP_ADDRESS, PORT))
    s.connect((IP_ADDRESS, PORT))
    s.send("[[current price]] {}".format(price).encode('utf-8'))
    print("Data sent, closing connection...")
    s.close()

print("Waiting for the beginning of next hour")
sleepUntil(NEXT_HOUR)

while True:
    try: 
        current_price = scraper.get_current_price()
     
        upload_to_server(current_price)

        print("Success! Going to sleep...")
    
        sleepUntil(NEXT_HOUR)
    except:
        print("Error occurred retrying in a minute")
        time.sleep(ONE_MINUTE)
