import {
  elementScroll,
  observeElementOffset,
  observeElementRect,
  Virtualizer,
} from "@tanstack/virtual-core";
import type { ComponentChildren, RefObject } from "preact";
import { useLayoutEffect, useRef, useState } from "preact/hooks";

/**
 * Renders only the items in and near the visible part of `scrollRef`, so a long history
 * stays fast. Items can be any height: each is measured when it renders, and re-measured
 * when it resizes.
 */
export function VirtualList<T>(props: {
  scrollRef: RefObject<HTMLElement | null>;
  items: T[];
  // Identifies an item across updates, so its measured height follows it when the list reorders.
  getKey: (item: T) => string | number;
  // A guess at an item's height until it has rendered.
  estimateHeight: (item: T) => number;
  children: (item: T) => ComponentChildren;
}) {
  const { items } = props;
  const listRef = useRef<HTMLDivElement>(null);
  const [, setVersion] = useState(0);
  // How far down the scroll container the list starts, i.e. the height of what's above it.
  const [offset, setOffset] = useState(0);

  const options = {
    count: items.length,
    getScrollElement: () => props.scrollRef.current,
    estimateSize: (index: number) => props.estimateHeight(items[index]),
    getItemKey: (index: number) => props.getKey(items[index]),
    overscan: 5,
    scrollMargin: offset,
    observeElementRect,
    observeElementOffset,
    scrollToFn: elementScroll,
    onChange: () => setVersion((v) => v + 1),
  };
  const [virtualizer] = useState(() => new Virtualizer<HTMLElement, HTMLDivElement>(options));
  virtualizer.setOptions(options);

  useLayoutEffect(() => {
    setOffset(listRef.current?.offsetTop ?? 0);
    return virtualizer._didMount();
  }, []);
  useLayoutEffect(() => virtualizer._willUpdate());

  return (
    <div ref={listRef} class="virtual-list" style={{ height: `${virtualizer.getTotalSize()}px` }}>
      {virtualizer.getVirtualItems().map((row) => (
        <div
          key={row.key}
          data-index={row.index}
          ref={virtualizer.measureElement}
          class="virtual-row"
          style={{ transform: `translateY(${row.start - offset}px)` }}
        >
          {props.children(items[row.index])}
        </div>
      ))}
    </div>
  );
}
