docker build -t status-service .
docker run -p 3000:3000 --name status-service-container -d status-service

# env variables

GET_MAIN_DOOR_STATUS_URL=http://127.0.0.1:3030/main-door-status
GET_BACK_DOOR_STATUS_URL=http://127.0.0.1:3030/back-door-status
