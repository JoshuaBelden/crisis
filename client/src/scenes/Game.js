import { Scene } from "phaser"
import { v4 as uuidv4 } from "uuid"

export class Game extends Scene {
    playerId = uuidv4()

    constructor() {
        super("Game")
    }

    async getWebSocketUrl() {
        try {
            const payload = {
                playerId: this.playerId,
                topics: ["world-events"],
            }
            const response = await fetch("http://localhost:8000/register", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify(payload),
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

        const serverUrl = await this.getWebSocketUrl()
        const connection = new WebSocket(serverUrl)

        connection.onopen = () => {
            const gameCommandRequest = {
                playerId: this.playerId,
                gameCommand: {
                    createUnit: {
                        unitId: uuidv4(),
                        position: [200, 200],
                    },
                },
            }
            connection.send(JSON.stringify(gameCommandRequest))
        }

        connection.onmessage = event => {
            const worldEventResponse = JSON.parse(event.data)
            if (worldEventResponse.worldEvent.unitCreated) {
                const [x, y] = worldEventResponse.worldEvent.unitCreated.position
                const ship = this.add
                    .image(x, y, "ship")
                    .setOrigin(0.5, 0.5)
            }
        }

        connection.onerror = error => {
            console.error("Error:", error)
        }

        this.input.once("pointerdown", () => {
            const gameCommandRequest
        })
    }
}
