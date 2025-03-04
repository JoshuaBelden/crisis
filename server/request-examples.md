curl -X POST 'http://localhost:8000/register' -H 'Content-Type: application/json' -d '{ "user_id": 1, "topic":"game-state" }'

websocat -t ws://127.0.0.1:8000/ws/51fcfc3232ff4dc0bb5e87c2cc234e81

curl -X DELETE 'http://localhost:8000/register/7ec113f459bb4f8d9ec1f9b3efde2793'

curl -X POST 'http://localhost:8000/publish' -H 'Content-Type: application/json' -d '{"user_id": 7, "topic": "game-state", "message": "{\"x\":300, \"y\":300}"}'
