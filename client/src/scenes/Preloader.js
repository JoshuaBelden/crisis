import { Scene } from "phaser"

export class Preloader extends Scene {
    constructor() {
        super("Preloader")
    }

    init() {}

    preload() {
        this.load.setPath('assets');
        this.load.image("ship", "spaceShips_001.png")
    }

    create() {
        this.scene.start("MainMenu")
    }
}
