import { Scene } from "phaser"
import { v4 as uuidv4 } from "uuid"

export class Game extends Scene {
    UNIT_SPEED = 200

    constructor() {
        super("Game")
        this.connection = null
        this.playerId = uuidv4()
        this.ships = []
    }

    async getWebSocketUrl() {
        try {
            const payload = {
                playerId: this.playerId,
                topics: ["debug", "world-events"],
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

    sendCreateUnit(initialPosition) {
        const gameCommandRequest = {
            playerId: this.playerId,
            gameCommand: {
                createUnit: {
                    unitId: uuidv4(),
                    position: initialPosition,
                },
            },
        }
        this.connection.send(JSON.stringify(gameCommandRequest))
    }

    sendMoveUnit(unitId, x, y) {
        const gameCommandRequest = {
            playerId: this.playerId,
            gameCommand: {
                moveUnit: {
                    unitId: unitId,
                    position: [x, y],
                },
            },
        }
        this.connection.send(JSON.stringify(gameCommandRequest))
    }

    createUnit(unitId, x, y) {
        const ship = this.add.image(x, y, "ship").setOrigin(0.5, 0.5)
        ship.setData("playerId", this.playerId)
        ship.setData("unitId", unitId)
        this.ships.push(ship)
    }

    moveUnit(unitId, x, y) {
        const ship = this.ships.find(ship => ship.getData("unitId") === unitId)
        ship.setData("destination", { x, y })
    }

    receiveMessage(data) {
        console.debug("Received message:", data)
        
        try {
            const worldEventResponse = JSON.parse(data)
            const eventType = Object.keys(worldEventResponse.worldEvent)[0]
            const eventData = worldEventResponse.worldEvent[eventType]
            switch (eventType) {
                case "unitCreated":
                    const [x, y] = eventData.position
                    this.createUnit(eventData.unitId, x, y)
                    break

                case "unitMoved":
                    const [moveX, moveY] = eventData.position
                    this.moveUnit(eventData.unitId, moveX, moveY)
                    break
            }
        } catch (err) {}
    }

    async create() {
        this.cameras.main.setBackgroundColor(0x00)

        const serverUrl = await this.getWebSocketUrl()
        this.connection = new WebSocket(serverUrl)
        this.connection.onopen = () => {
            const initialRandomPosition = [
                Math.floor(Math.random() * this.game.canvas.clientWidth),
                Math.floor(Math.random() * this.game.canvas.clientHeight),
            ]
            this.sendCreateUnit(initialRandomPosition)
        }

        this.connection.onmessage = event => {
            this.receiveMessage(event.data)
        }

        this.connection.onerror = error => {
            console.error("Error:", error)
        }

        this.input.on("pointerdown", pointer => {
            const myShipId = this.ships
                .find(ship => ship.getData("playerId") === this.playerId)
                .getData("unitId")
            const x = pointer.x
            const y = pointer.y
            this.sendMoveUnit(myShipId, x, y)
        })
    }

    async update() {
        if (!this.ships.length) {
            return
        }

        this.ships.forEach(ship => {
            const destination = ship.getData("destination")
            if (!destination) {
                return
            }

            const distance = Phaser.Math.Distance.Between(
                ship.x,
                ship.y,
                destination.x,
                destination.y
            )

            if (distance < 1) {
                ship.setData("destination", null)
                return
            }
            const duration = (distance / this.UNIT_SPEED) * 1000 // duration in milliseconds

            if (distance > 1) {
                this.tweens.add({
                    targets: ship,
                    x: destination.x,
                    y: destination.y,
                    duration: duration,
                    ease: "Linear",
                })
                ship.setData("destination", null)
            }
        })
    }
}
