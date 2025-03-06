import { Scene } from "phaser"
import { v4 as uuidv4 } from "uuid"

export class Game extends Scene {
    ENTITY_SPEED = 200

    constructor() {
        super("Game")
        this.connection = null
        this.playerId = uuidv4()
        this.entities = []
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

    sendListEntities() {
        const worldCommandRequest = {
            playerId: this.playerId,
            worldCommand: {
                listEntities: null,
            },
        }
        this.connection.send(JSON.stringify(worldCommandRequest))
    }

    sendCreateEntity(initialPosition) {
        const worldCommandRequest = {
            playerId: this.playerId,
            worldCommand: {
                createEntity: {
                    entityId: uuidv4(),
                    position: initialPosition,
                },
            },
        }
        this.connection.send(JSON.stringify(worldCommandRequest))
    }

    sendMoveEntity(entityId, x, y) {
        const gameCommandRequest = {
            playerId: this.playerId,
            worldCommand: {
                moveEntity: {
                    entityId,
                    position: [x, y],
                },
            },
        }
        this.connection.send(JSON.stringify(gameCommandRequest))
    }

    createEntity(entityId, x, y) {
        const entity = this.add.image(x, y, "ship").setOrigin(0.5, 0.5)
        entity.setData("playerId", this.playerId)
        entity.setData("entityId", entityId)
        this.entities.push(entity)
    }

    moveEntity(entityId, x, y) {
        const entity = this.entities.find(entity => {
            return entity.data.values.playerId === this.playerId
        })
        if (!entity) {
            return
        }

        entity.setData("destination", { x, y })
        console.log("Entity destination set to:", entity.getData("destination"))
    }

    receiveMessage(data) {
        console.debug("Received message:", data)

        try {
            const worldEventResponse = JSON.parse(data)
            const eventType = Object.keys(worldEventResponse.worldEvent)[0]
            const eventData = worldEventResponse.worldEvent[eventType]
            switch (eventType) {
                case "entitiesListed":
                    eventData.entities.forEach(entity => {
                        if (
                            this.entities.some(
                                e => e.data.values.entityId === entity.entityId
                            )
                        ) {
                            return
                        }

                        this.createEntity(
                            entity.entityId,
                            entity.position[0],
                            entity.position[1]
                        )
                    })
                    break

                case "entityCreated":
                    const [x, y] = eventData.position
                    this.createEntity(eventData.entityId, x, y)
                    break

                case "entityMoved":
                    const [moveX, moveY] = eventData.position
                    this.moveEntity(eventData.entityId, moveX, moveY)
                    break
            }
        } catch (err) {}
    }

    async create() {
        this.cameras.main.setBackgroundColor(0x00)
        this.add.text(10, 10, `PlayerId: ${this.playerId}`, {
            font: "16px Courier",
            fill: "#00ff00",
        })

        const serverUrl = await this.getWebSocketUrl()
        this.connection = new WebSocket(serverUrl)
        this.connection.onopen = () => {
            this.sendListEntities()

            const initialRandomPosition = [
                Math.floor(Math.random() * this.game.canvas.clientWidth),
                Math.floor(Math.random() * this.game.canvas.clientHeight),
            ]
            this.sendCreateEntity(initialRandomPosition)
        }

        this.connection.onmessage = event => {
            this.receiveMessage(event.data)
        }

        this.connection.onerror = error => {
            console.error("Error:", error)
        }

        this.input.on("pointerdown", pointer => {
            const playersEntity = this.entities.find(
                entity => entity.data.values.playerId === this.playerId
            )

            const x = pointer.x
            const y = pointer.y
            this.sendMoveEntity(playersEntity.data.values.entityId, x, y)
        })
    }

    async update() {
        if (!this.entities.length) {
            return
        }

        this.entities.forEach(entity => {
            const destination = entity.data.values.destination
            if (!destination) {
                return
            }

            const distance = Phaser.Math.Distance.Between(
                entity.x,
                entity.y,
                destination.x,
                destination.y
            )

            if (distance < 1) {
                entity.setData("destination", null)
                return
            }
            const duration = (distance / this.ENTITY_SPEED) * 1000 // duration in milliseconds

            if (distance > 1) {
                this.tweens.add({
                    targets: entity,
                    x: destination.x,
                    y: destination.y,
                    duration: duration,
                    ease: "Linear",
                })
                entity.setData("destination", null)
            }
        })
    }
}
