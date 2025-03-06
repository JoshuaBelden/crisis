import { Scene } from "phaser"
import { v4 as uuidv4 } from "uuid"

export class Game extends Scene {
    constructor() {
        super("Game")
        this.connection = null
        this.playerId = uuidv4()
        this.ship = null
        this.destination = null
        this.speed = 200
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

    sendCreateUnit() {
        const gameCommandRequest = {
            playerId: this.playerId,
            gameCommand: {
                createUnit: {
                    unitId: uuidv4(),
                    position: [200, 200],
                },
            },
        }
        this.connection.send(JSON.stringify(gameCommandRequest))
    }

    sendMoveUnit(x, y) {
        const gameCommandRequest = {
            playerId: this.playerId,
            gameCommand: {
                moveUnit: {
                    unitId: uuidv4(),
                    position: [x, y],
                },
            },
        }
        this.connection.send(JSON.stringify(gameCommandRequest))
    }

    receiveMessage(data) {
        const worldEventResponse = JSON.parse(data)
        if (worldEventResponse.worldEvent.unitCreated) {
            const [x, y] = worldEventResponse.worldEvent.unitCreated.position
            this.ship = this.add.image(x, y, "ship").setOrigin(0.5, 0.5)
            return
        }

        if (worldEventResponse.worldEvent.unitMoved) {
            const [x, y] = worldEventResponse.worldEvent.unitMoved.position
            this.destination = { x, y }
            return
        }
    }

    async create() {
        this.cameras.main.setBackgroundColor(0x00)

        const serverUrl = await this.getWebSocketUrl()
        this.connection = new WebSocket(serverUrl)
        this.connection.onopen = () => {
            this.sendCreateUnit()
        }

        this.connection.onmessage = event => {
            this.receiveMessage(event.data)
        }

        this.connection.onerror = error => {
            console.error("Error:", error)
        }

        this.input.on("pointerdown", pointer => {
            const x = pointer.x
            const y = pointer.y
            this.sendMoveUnit(x, y)
        })
    }

    async update() {
        if (!this.ship || !this.destination) {
            return
        }

        const distance = Phaser.Math.Distance.Between(
            this.ship.x,
            this.ship.y,
            this.destination.x,
            this.destination.y
        )
        const duration = (distance / this.speed) * 1000 // duration in milliseconds

        if (distance > 1) {
            this.tweens.add({
                targets: this.ship,
                x: this.destination.x,
                y: this.destination.y,
                duration: duration,
                ease: "Linear",
            })
            this.destination = null // Clear the destination after starting the tween
        }
    }
}
