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

class TVModeNavigator {
  private rootNode: Ref<HTMLElement | undefined>;
  private navigationNodes: Map<string, Array<Element>> = new Map();

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
  }

  recursivelyFindInteractable(element: Element): Array<Element> {
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
    for (const element of interactiveElements) {
      const currentId = self.getInteractionId(element);
      const directionJumps: Map<Direction, NavigationJump> = new Map();
      for (const direction of Directions) {
        const jump = self.getNavJump(element, direction);
        if (jump) {
          directionJumps.set(direction, jump);
        }
      }

      const lowerX = element.clientLeft;
      const upperX = element.clientLeft + element.clientWidth;
      const lowerY = element.clientTop;
      const upperY = element.clientTop + element.clientHeight;

      for (const otherElement of interactiveElements) {
        const otherId = self.getInteractionId(otherElement);
        if (otherId == currentId) continue; // Skip us

        const otherLowerX = otherElement.clientLeft;
        const otherUpperX = otherElement.clientLeft + otherElement.clientWidth;
        const otherLowerY = otherElement.clientTop;
        const otherUpperY = otherElement.clientTop + otherElement.clientHeight;

        for (const direction of Directions) {
          let jump;
          switch (direction) {
            case "down":
              const leeway =
                0.1 * (element.clientWidth + otherElement.clientWidth);
              if (
                lowerX - leeway < otherUpperX ||
                upperX + leeway > otherLowerX
              )
                break;
              jump = {
                id: otherId,
                distance: upperY - otherUpperY,
              } satisfies NavigationJump;
              console.log(jump);
              break;
          }
        }
      }
    }
  }
}

export const createTVNavigator = (rootNode: Ref<HTMLElement | undefined>) =>
  new TVModeNavigator(rootNode);
