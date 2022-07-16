importScripts("https://unpkg.com/comlink/dist/umd/comlink.js");
importScripts("./pkg/boardopt_wasm.js");

wasm_bindgen("./pkg/boardopt_wasm_bg.wasm").then(() => {

    const worker_obj = {
        score: 0.0,
        placed: '',
        boosted: '',
        boardopt(boards, lms1, lms2, min_num_lm, max_num_lm, weights, booster) {
            let result = wasm_bindgen.boardopt(boards, lms1, lms2, min_num_lm, max_num_lm, weights, booster);
            console.log('WORKER:', result);
            this.score = result.score;
            this.placed = result.placed;
            this.boosted = result.boosted;
        },
    };

    Comlink.expose(worker_obj);

});