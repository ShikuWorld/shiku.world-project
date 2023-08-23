docker build -t status-service .
docker run -p 3000:3000 --name status-service-container -d status-service
