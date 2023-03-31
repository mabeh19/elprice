import time
import socket
import threading
import scheduler
import scraper
#import server_restart

def get_ip():
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        # doesn't even have to be reachable
        s.connect(('10.255.255.255', 1))
        IP = s.getsockname()[0]
    except:
        IP = '127.0.0.1'
    finally:
        s.close()
    return IP

ONE_MINUTE = 60
ONE_HOUR = 60 * 60
NEXT_HOUR = 1
IP_ADDRESS = get_ip()
#hostname = socket.gethostname()
#IP_ADDRESS = socket.gethostbyname(hostname)
PORT = 35000


def upload_to_server(price):
    print("Creating socket...")
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    print("Connecting to {}:{}".format(IP_ADDRESS, PORT))
    s.connect((IP_ADDRESS, PORT))
    s.send("[[current price]] {}".format(price).encode('utf-8'))
    print("Data sent, closing connection...")
    s.close()

#threading.Thread(target=server_restart.serverThread).start()

print("Waiting for the beginning of next hour")
scheduler.sleepUntilNext(NEXT_HOUR)

while True:
    try: 
        current_price = scraper.get_current_price()
    
        if current_price != -1:
            upload_to_server(current_price)
            print("Success! Going to sleep...")
    
        scheduler.sleepUntilNext(NEXT_HOUR)
    except:
        print("Error occurred retrying in a minute")
        time.sleep(ONE_MINUTE)
