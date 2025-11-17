const NAVIGATE_MODIFIED_PROP = "tvnav-id";
const NAVIGATE_INTERACT_ID = "tvnav-iid";
const NAVIGATE_DEBUG_TAG = "tvnavdebug";

const Directions = ["left", "right", "up", "down"] as const;
type Direction = (typeof Directions)[number];

interface NavigationJump {
  distance: number;
  id: string;
}

type Position = [number, number, number, number, string];

class TVModeNavigator {
  private navigationNodes: Map<string, HTMLElement> = new Map();

  constructor() {
    const thisRef = this;
    const observer = new MutationObserver((v, k) => {
      this.onMutation(thisRef, v, k);
    });
    observer.observe(document.getRootNode(), {
      childList: true,
      subtree: true,
    });

    document.addEventListener("keydown", (ev) => {
      switch (ev.code) {
        case "KeyW":
          return this.moveUp();
        case "KeyS":
          return this.moveDown();
        case "KeyD":
          return this.moveRight();
        case "KeyA":
          return this.moveLeft();
      }
    });
  }

  private getCurrentPosition(element?: HTMLElement): Position {
    const el = element || document.activeElement;
    if (!el) throw "No active position";
    const rect = el.getBoundingClientRect();
    const debugName = el.getAttribute(NAVIGATE_DEBUG_TAG);
    return [rect.left, rect.right, rect.top, rect.bottom, debugName ?? ""];
  }

  private isSamePosition(a: Position, b: Position) {
    return a.map((v, i) => v === b[i]).every((v) => v);
  }

  private getUniqueNavNodes() {
    const hasSeen = new Map<string, boolean>();
    return this.navigationNodes.values().filter((v) => {
      const id = this.getInteractionId(v);
      if (hasSeen.get(id)) return false;
      hasSeen.set(id, true);
      return true;
    });
  }

  /*
    let left = current[0] - target[1]; // Our left edge vs their right left (they're to our left)
    if (left < 0) {
      left = target[0] - current[1];
      console.log("to the right");
    } // If we're less, it's our right edge vs their left edge
    
    let top = current[2] - target[3]; // Our top edge vs their bottom edge
    if (top < 0) {
      top = target[2] - current[3];
      console.log("undernearth");
    } // Our bottom edge vs their top edge
    console.log(top);
  */

  private getCenter(position: Position) {
    return [
      position[0] + 0.5 * (position[1] - position[0]),
      position[2] + 0.5 * (position[3] - position[2]),
    ];
  }

  private findOuterDistance(
    current: Position,
    target: Position,
    bias: Direction
  ) {
    const centerCurrent = this.getCenter(current);
    const centerTarget = this.getCenter(target);

    return Math.sqrt(
      Math.pow(
        centerCurrent[0] - centerTarget[0],
        ["left", "right"].includes(bias) ? 2 : 4
      ) +
        Math.pow(
          centerCurrent[1] - centerTarget[1],
          ["up", "down"].includes(bias) ? 2 : 4
        )
    );
  }

  private findElementWithPredicate(
    check: (current: Position, target: Position) => boolean,
    direction: Direction
  ) {
    const current = this.getCurrentPosition();
    // We want things in the x direction, with a limit on the y
    let distance = Infinity;
    let element = null;
    const nodes = this.getUniqueNavNodes();
    for (const newElement of nodes) {
      const target = this.getCurrentPosition(newElement);
      if (this.isSamePosition(current, target)) continue;
      if (target.every((v) => v == 0)) continue; // This element doesn't exist
      const newDistance = this.findOuterDistance(current, target, direction);
      console.log(
        `distance of ${newDistance} between ${current[4]} and ${target[4]}`
      );
      // If we're the wrong way, or further than the current option
      if (newDistance < 0 || newDistance > distance) {
        continue;
      }

      if (!check(current, target)) {
        continue;
      }
      distance = newDistance;
      element = newElement;
    }
    return element;
  }

  moveUp() {
    const leeway = 100; // 20px
    const element = this.findElementWithPredicate(
      ([xleft, xright, ytop, ybottom], [eleft, eright, etop, ebottom]) =>
        xleft - leeway < eright && xright + leeway > eleft && ytop >= ebottom,
      "up"
    );

    if (element) {
      element.focus();
    }
  }

  moveDown() {
    const leeway = 20; // 20px
    const element = this.findElementWithPredicate(
      ([xleft, xright, ytop, ybottom], [eleft, eright, etop, ebottom]) =>
        xleft - leeway < eright && xright + leeway > eleft && ytop <= ebottom,
      "down"
    );

    if (element) {
      element.focus();
    }
  }

  moveRight() {
    const leeway = 0; // 20px
    const element = this.findElementWithPredicate(
      ([xleft, xright, ytop, ybottom], [eleft, eright, etop, ebottom]) =>
        ytop - leeway < ebottom && ybottom + leeway > etop && xright <= eleft,
      "right"
    );

    if (element) {
      element.focus();
    }
  }

  moveLeft() {
    const leeway = 20; // 20px
    const element = this.findElementWithPredicate(
      ([xleft, xright, ytop, ybottom], [eleft, eright, etop, ebottom]) =>
        ytop - leeway < ebottom && ybottom + leeway > etop && xleft >= eright,
      "left"
    );

    if (element) {
      element.focus();
    }
  }

  recursivelyFindInteractable(element: Element): Array<HTMLElement> {
    const elements = [];
    for (const child of element.children) {
      if (!child) continue;
      if (child instanceof HTMLAnchorElement) {
        elements.push(child);
        continue;
      }
      if (child instanceof HTMLButtonElement) {
        elements.push(child);
        continue;
      }
      if (child instanceof HTMLInputElement) {
        elements.push(child);
        continue;
      }

      // Save ourselves a function call
      if (child.children.length > 0) {
        elements.push(...this.recursivelyFindInteractable(child));
      }
    }
    return elements;
  }

  getInteractionId(element: Element) {
    const id = element.getAttribute(NAVIGATE_INTERACT_ID);
    if (id) return id;
    const newId = crypto.randomUUID();
    element.setAttribute(NAVIGATE_INTERACT_ID, newId);
    return newId;
  }

  onMutation(
    self: TVModeNavigator,
    mutationlist: Array<MutationRecord>,
    observer: unknown
  ) {
    for (const mutation of mutationlist) {
      for (const node of mutation.addedNodes) {
        if (!node) continue;
        if (node.nodeType !== Node.ELEMENT_NODE) continue;
        const el = node as Element;

        const interactiveNodes = self.recursivelyFindInteractable(el);

        for (const v of interactiveNodes) {
          const id = self.getInteractionId(v);
          self.navigationNodes.set(id, v);
        }
      }
    }

    const interactiveElements = this.navigationNodes.values().toArray().flat();

    // Set focus so we aren't confused
    if (!document.activeElement || document.activeElement.tagName === "BODY") {
      const active = interactiveElements.at(0);
      if (active) {
        active.focus();
      }
    }
  }
}

export const createTVNavigator = () => new TVModeNavigator();
