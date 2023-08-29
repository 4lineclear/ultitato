export function generateBoard(): void {
    let game = document.createElement("div");
    game.className = "container";
    for (let rows = 0; rows < 3; rows++) {
        let row = document.createElement("div");
        row.className = "row";
        for (let i = 1; i <= 3; i++) {
            let col = document.createElement("div");
            col.className = "col";
            for (let j = 0; j < 3; j++) {
                let row = document.createElement("div");
                row.className = "row";
                for (let k = 0; k < 3; k++) {
                    let col = document.createElement("div");
                    col.className = "col";
                    col.innerText = String(j * 3 + k);
                    row.appendChild(col);
                }
                col.appendChild(row);
            }
            row.appendChild(col);
        }
        game.appendChild(row);
    }
    document.getElementById("game-container").appendChild(game);
}
