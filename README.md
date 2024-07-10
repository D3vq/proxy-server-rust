#Rust Proxy Server with Docker
#This repository contains a Rust-based proxy server that caches and forwards HTTP requests securely. It is Dockerized for easy deployment.

#Usage
#Build the Docker Image
1) Clone the repository:  
  git clone <repository-url>
  cd <repository-directory>

2)Build the Docker image:
  docker build -t proxy-server .

#Run the Docker Container
3)Run the Docker container with the desired credentials (replace your_username:your_password with your actual credentials):
  docker run -p 8080:8080 -e PROXY_CREDENTIALS="your_username:your_password" proxy-server

example:
  docker run -p 8080:8080 -e PROXY_CREDENTIALS="sumit:123" proxy-server

This command starts the proxy server inside a Docker container, exposing port 8080 on your local machine.

#Test with curl
4)In another terminal, test the proxy server using curl:
  curl -H "Authorization: Basic c3VtaXQ6MTIz" http://localhost:8080/https://www.google.com

Here, c3VtaXQ6MTIz is the base64-encoded string of sumit:123 (which is your_username:your_password). Replace it with your own base64-encoded credentials.

#Notes
The proxy server runs on localhost:8080.
Ensure Docker is properly installed and running on your machine.
Adjust firewall settings if necessary to allow traffic on port 8080.


