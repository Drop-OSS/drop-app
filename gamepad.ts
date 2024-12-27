import gameControl, { XBoxButton, Button, type GCGamepad } from 'esm-gamecontroller.js';

let mainGamepad: Ref<number | undefined>  = ref(undefined);

let buttonIndex = 0;
const buttons = computed((): any[]  => {
    let as = document.getElementsByTagName('a');
    let buttons = document.getElementsByTagName('button');
    return [].concat(Array.from(as)).concat(Array.from(buttons));
})

const wrap = (num: number, min: number, max: number) => ((((num - min) % (max - min)) + (max - min)) % (max - min)) + min;

setInterval(() => {
    mainGamepad.value = navigator.getGamepads().filter(g => g !== null)[0]?.axes[1];
    console.log(navigator.getGamepads().filter(g => g !== null)[0]?.axes)
  }, 100);

watch(mainGamepad, (v) => {
  console.log(v)
  if (!v || v == 0) return;
  buttonIndex = wrap(buttonIndex + v > 0 ? 1 : -1, 0, buttons.value.length);

  console.log(`Focusing ${buttonIndex}`)
  console.log(buttons.value[buttonIndex]);
  
  buttons.value[buttonIndex].focus()
})



/*
setInterval(() => {
    console.log(gamepads[0]);
    console.log(gamepads[0].checkStatus())
}, 1000);
*/

/*
window.ongamepadconnected = (e) => {
    console.log("ongamepadconnected", e);
}


function selectButton(index: number) {
    buttonIndex = (buttonIndex + index) % buttons.value.length; 
    console.log(`Selecting button ${buttonIndex}`);
    buttons.value[buttonIndex].focus()
}

function processGamepads() {
    while (true) {
        console.log("Processing gamepads");
        let gamepad = navigator.getGamepads()[0];
        if (!gamepad) continue;
        let direction = gamepad.axes[1];
        if (direction > 0.1) {
            selectButton(1)
            console.log("Selecting button 1")
        }
        else if (direction < -0.1) {
            selectButton(-1)
            console.log("Selecting button -1")
        }
    }
}
*/