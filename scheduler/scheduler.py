import time, datetime

def sleepUntilNext(hour, minute=0):
    t = datetime.datetime.today()
    h = t.hour + hour
    future = datetime.datetime(t.year, t.month, t.day, t.hour, minute)
#    if h > 23:
#        future += datetime.timedelta(days=1)
    future += datetime.timedelta(hours=hour)
    print("Sleeping until ", future)
    time.sleep((future-t).total_seconds())

def sleepUntil(hour, minute=0):
    t = datetime.datetime.today()
    future = datetime.datetime(t.year, t.month, t.day, hour, minute)
    if t.timestamp() > future.timestamp():
        future += datetime.timedelta(days=1)
    time.sleep((future-t).total_seconds())
