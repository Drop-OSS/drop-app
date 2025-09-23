const NAVIGATE_MODIFIED_PROP = "tvnav-id";
const NAVIGATE_INTERACT_ID = "tvnav-iid";

const Directions = ["left", "right", "up", "down"] as const;
type Direction = (typeof Directions)[number];

const NAVIGATE_LEFT_ID = "tvnav-left";
const NAVIGATE_RIGHT_ID = "tvnav-right";
const NAVIGATE_UP_ID = "tvnav-up";
const NAVIGATE_DOWN_ID = "tvnav-down";

interface NavigationJump {
  distance: number;
  id: string;
}

type Position = [number, number, number, number];

class TVModeNavigator {
  private rootNode: Ref<HTMLElement | undefined>;
  private navigationNodes: Map<string, Array<HTMLElement>> = new Map();

  constructor(rootNode: Ref<HTMLElement | undefined>) {
    this.rootNode = rootNode;

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

  private findElementWithPredicate(
    distanceCalculator: (current: Position, target: Position) => number,
    check: (current: Position, target: Position) => boolean
  ) {
    const current = this.getCurrentPosition();
    // We want things in the x direction, with a limit on the y
    let distance = Math.max(window.innerWidth, window.innerHeight);
    let element = null;
    for (const newElement of this.navigationNodes.values().toArray().flat()) {
      const target = this.getCurrentPosition(newElement);
      if(this.isSamePosition(current, target)) continue;
      const newDistance = distanceCalculator(current, target);
      // If we're the wrong way, or further than the current option
      if (newDistance < 0 || newDistance > distance) continue;
      if (check(current, target)) continue;
      distance = newDistance;
      element = newElement;
    }
    return element;
  }

  moveUp() {
    const leeway = 20; // 20px
    const element = this.findElementWithPredicate(
      ([xleft, xright, ytop, ybottom], [eleft, eright, etop, ebottom]) =>
        ytop - ebottom,
      ([xleft, xright, ytop, ybottom], [eleft, eright, etop, ebottom]) =>
        xleft - leeway > eright && xright + leeway < eleft
    );

    if (element) {
      element.focus();
    }
  }

  moveDown() {
    const leeway = 20; // 20px
    const element = this.findElementWithPredicate(
      ([xleft, xright, ytop, ybottom], [eleft, eright, etop, ebottom]) =>
        etop - ybottom,
      ([xleft, xright, ytop, ybottom], [eleft, eright, etop, ebottom]) =>
        xleft - leeway > eright && xright + leeway < eleft
    );

    if (element) {
      element.focus();
    }
  }

  moveRight() {
    const leeway = 20; // 20px
    const element = this.findElementWithPredicate(
      ([xleft, xright, ytop, ybottom], [eleft, eright, etop, ebottom]) =>
        eleft - xright,
      ([xleft, xright, ytop, ybottom], [eleft, eright, etop, ebottom]) =>
        etop - leeway > ytop && ebottom + leeway < etop
    );

    if (element) {
      element.focus();
    }
  }

  moveLeft() {
    const leeway = 20; // 20px
    const element = this.findElementWithPredicate(
      ([xleft, xright, ytop, ybottom], [eleft, eright, etop, ebottom]) =>
        xleft - eright,
      ([xleft, xright, ytop, ybottom], [eleft, eright, etop, ebottom]) =>
        etop - leeway > ytop && ebottom + leeway < etop
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
      if(child instanceof HTMLInputElement) {
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

  private getNavJumpKey(direction: Direction) {
    switch (direction) {
      case "down":
        return NAVIGATE_DOWN_ID;
      case "left":
        return NAVIGATE_LEFT_ID;
      case "right":
        return NAVIGATE_RIGHT_ID;
      case "up":
        return NAVIGATE_UP_ID;
    }

    throw "Invalid direction";
  }

  getNavJump(
    element: Element,
    direction: Direction
  ): NavigationJump | undefined {
    const key = this.getNavJumpKey(direction);
    const value = element.getAttribute(key);
    if (!value) return undefined;
    const [id, distance] = value.split("/");
    return {
      distance: parseFloat(distance),
      id,
    };
  }

  onMutation(
    self: TVModeNavigator,
    mutationlist: Array<MutationRecord>,
    observer: unknown
  ) {
    for (const mutation of mutationlist) {
      for (const node of mutation.removedNodes) {
        if (node.nodeType !== Node.ELEMENT_NODE) continue;
        const el = node as Element;
        const id = el.getAttribute(NAVIGATE_MODIFIED_PROP);
        if (id) {
          self.navigationNodes.delete(id);
        }
      }

      for (const node of mutation.addedNodes) {
        if (!node) continue;
        if (node.nodeType !== Node.ELEMENT_NODE) continue;
        const el = node as Element;

        const existingId = el.getAttribute(NAVIGATE_MODIFIED_PROP);
        if (existingId) {
          self.navigationNodes.delete(existingId);
        }

        const interactiveNodes = self.recursivelyFindInteractable(el);

        const id = crypto.randomUUID();
        el.setAttribute(NAVIGATE_MODIFIED_PROP, id);

        self.navigationNodes.set(id, interactiveNodes);
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

export const createTVNavigator = (rootNode: Ref<HTMLElement | undefined>) =>
  new TVModeNavigator(rootNode);
