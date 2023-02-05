import datetime
import subprocess
import scheduler
import docker

resetTime = datetime.time(4, 30, 0) # restart server at 4:30 am
client = docker.from_env()
updateRepoCommand = "cd ../ && git pull"

def updateRepo():
    subprocess.run(updateRepoCommand)

def runServer():
    return client.containers.run("price_server")

def serverThread():
    while True:
        server = runServer()
        scheduler.sleepUntil(resetTime.hour, resetTime.minute)
        server.kill()
        updateRepo()
