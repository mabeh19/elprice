#include <stdio.h>
#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <sys/wait.h>
#include <string.h>
#include <arpa/inet.h>
#include <sys/socket.h>
#include <time.h>

#define SERVER_PORT	35000
#define FD_READ 0
#define FD_WRITE 1

float GetPrice(void)
{
char* curl_cmd_arg = "https://api.energifyn.dk/api/graph/consumptionprice?date=";
int fds[2];
int pid;
char buff[0x1024] = { 0 };
int i = 0;
char* token;
char* word = "\"currentWestPowerPrice\":";
	
	time_t t = time(NULL);
	struct tm date = *localtime(&t);
	
	pipe(fds);

	if ((pid = fork()) == 0) {
		asprintf(&curl_cmd_arg, "%s%02d-%02d-%04d", curl_cmd_arg, date.tm_mday, date.tm_mon + 1, date.tm_year + 1900);

		char* args[] = {
			"curl",
			"-s", 			// Run in silent mode
			curl_cmd_arg,
			NULL
		};

		dup2(fds[FD_WRITE], STDOUT_FILENO);
		close(fds[FD_READ]);
		execvp(args[0], args);

		free(curl_cmd_arg); 	// If we failed to execute program, free our data
	} else {
		close(fds[FD_WRITE]);
		waitpid(pid, NULL, 0); // wait for child to finish
	}

	while (read(fds[FD_READ], &buff[i++], 1) > 0); // fill buffer with gotten data
	
	close(fds[FD_READ]);
	
	token = strstr(buff, word) + strlen(word);

	return atof(token);
}

void SendToServer(float price)
{
struct sockaddr_in serverAddress;
int clientFD, status;
char* serverCommand;

	printf("Establishing connection to server...\n");
	if ((clientFD = socket(AF_INET, SOCK_STREAM, 0)) < 0) {
		printf("Socket creation failed!\n");
		exit(EXIT_FAILURE);
	}

	serverAddress.sin_family = AF_INET;
	serverAddress.sin_port = htons(SERVER_PORT);

	if (inet_pton(AF_INET, "192.168.87.188", &serverAddress.sin_addr) <= 0) {
		printf("Invalid address\n");
		exit(EXIT_FAILURE);
	}

	if ((status = connect(clientFD, (struct sockaddr*)&serverAddress, sizeof(serverAddress))) < 0) {
		printf("Unable to establish connection to server\n");
		exit(EXIT_FAILURE);
	}

	printf("Connected!\n");

	/* Build command */
	asprintf(&serverCommand, "[[current price]] %f\r\n", price);
	/* Tell server the current price */
	send(clientFD, serverCommand, strlen(serverCommand), 0);
	close(clientFD);
	printf("Connection closed\n");
	free(serverCommand);	// Remember to free your data kids
}

int SecondsToNextHour(void)
{
	time_t t = time(NULL);
	struct tm date = *localtime(&t);
	
	return 3600 - (60 * date.tm_min + date.tm_sec);
}

void Run(void)
{
	for (;;) {
		float price = GetPrice();
		printf("Current price = %.2f\n", price);
		SendToServer(price);
		printf("Sleep time\n");
		sleep((float)SecondsToNextHour());
	}
}

int main()
{
	Run();
	return EXIT_SUCCESS;
}
