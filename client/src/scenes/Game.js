import { Scene } from "phaser"

export class Game extends Scene {
    constructor() {
        super("Game")
    }

    async getWebSocketUrl() {
        try {
            const response = await fetch("http://localhost:8000/register", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({ player_id: 7, topic: "game-state" }),
            })

            if (!response.ok) {
                throw new Error("Network response was not ok")
            }

            const data = await response.json()
            return data.url
        } catch (err) {
            console.error("Error:", err)
        }
    }

    async create() {
        this.cameras.main.setBackgroundColor(0x00)
        const ship = this.add.image(200, 200, "ship").setOrigin(0.5, 0.5)
        
        const serverUrl = await this.getWebSocketUrl()
        const connection = new WebSocket(serverUrl)

        connection.onopen = () => {
            console.log("Connected to the server")
        }

        connection.onmessage = event => {
            console.log("Message from server ", event.data)
            const {x, y} = JSON.parse(event.data);
            ship.setPosition(x, y);
        }

        connection.onerror = error => {
            console.error("Error:", error)
        }

        connection.send(JSON.stringify({ x: 200, y: 200 }))

        this.input.once("pointerdown", () => {
            this.scene.start("GameOver")
        })
    }
}
