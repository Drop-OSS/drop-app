const NAVIGATE_MODIFIED_PROP = "tvnav-id";
const NAVIGATE_INTERACT_ID = "tvnav-iid";

const Directions = ["left", "right", "up", "down"] as const;
type Direction = (typeof Directions)[number];

interface NavigationJump {
  distance: number;
  id: string;
}

type Position = [number, number, number, number];

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
    return [rect.left, rect.right, rect.top, rect.bottom];
  }

  private isSamePosition(a: Position, b: Position) {
    return a.map((v, i) => v === b[i]).every((v) => v);
  }

  private getUniqueNavNodes() {
    const hasSeen = new Map<string, boolean>();
    return this.navigationNodes
      .values()
      .filter((v) => {
        const id = this.getInteractionId(v);
        if (hasSeen.get(id)) return false;
        hasSeen.set(id, true);
        return true;
      });
  }

  private findElementWithPredicate(
    check: (current: Position, target: Position) => boolean
  ) {
    const current = this.getCurrentPosition();
    // We want things in the x direction, with a limit on the y
    let distance = Math.max(window.innerWidth, window.innerHeight);
    let element = null;
    const nodes = this.getUniqueNavNodes();
    for (const newElement of nodes) {
      const target = this.getCurrentPosition(newElement);
      if (this.isSamePosition(current, target)) continue;
      const newDistance = Math.sqrt(
        Math.pow(current[0] - target[0], 2) +
          Math.pow(current[2] - target[2], 2)
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
        xleft - leeway < eright && xright + leeway > eleft && ytop >= ebottom
    );

    if (element) {
      element.focus();
    }
  }

  moveDown() {
    const leeway = 20; // 20px
    const element = this.findElementWithPredicate(
      ([xleft, xright, ytop, ybottom], [eleft, eright, etop, ebottom]) =>
        xleft - leeway < eright && xright + leeway > eleft && ytop <= ebottom
    );

    if (element) {
      element.focus();
    }
  }

  moveRight() {
    const leeway = 0; // 20px
    const element = this.findElementWithPredicate(
      ([xleft, xright, ytop, ybottom], [eleft, eright, etop, ebottom]) =>
        ytop - leeway < ebottom && ybottom + leeway > etop && xright <= eleft
    );

    if (element) {
      element.focus();
    }
  }

  moveLeft() {
    const leeway = 20; // 20px
    const element = this.findElementWithPredicate(
      ([xleft, xright, ytop, ybottom], [eleft, eright, etop, ebottom]) =>
        ytop - leeway < ebottom && ybottom + leeway > etop && xleft >= eright
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
