import("./pkg").then(module => {
    let petriControlElements = document.getElementsByClassName('petricontrols');
    for (i = 0; i < petriControlElements.length; i++) {
        let e = petriControlElements[i];
        try {
            let settings = e.dataset.petricontrols;
            module.run_app(e.id, settings);
        } catch (error) {
            console.error(error);
        }
    }
});
