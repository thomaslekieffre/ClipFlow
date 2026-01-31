import {
  DndContext,
  closestCenter,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  horizontalListSortingStrategy,
  arrayMove,
} from "@dnd-kit/sortable";
import { useAppStore } from "../../stores/appStore";
import { SortableClipCard } from "./SortableClipCard";
import { TransitionIcon } from "./TransitionIcon";

export function Timeline() {
  const { clips, transitions, reorderClips, deleteClip, setTransition, setClipTrim } =
    useAppStore();

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: { distance: 5 },
    }),
  );

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (!over || active.id === over.id) return;

    const oldIndex = clips.findIndex((c) => c.id === active.id);
    const newIndex = clips.findIndex((c) => c.id === over.id);
    const newOrder = arrayMove(clips, oldIndex, newIndex);
    reorderClips(newOrder.map((c) => c.id));
  };

  const totalDuration = clips.reduce((sum, c) => sum + c.duration_ms, 0);

  return (
    <div className="w-full">
      {/* Total duration */}
      <div className="text-xs text-zinc-500 dark:text-zinc-500 mb-3 px-1">
        {clips.length} clip{clips.length > 1 ? "s" : ""} ·{" "}
        {(totalDuration / 1000).toFixed(1)}s total
      </div>

      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragEnd={handleDragEnd}
      >
        <SortableContext
          items={clips.map((c) => c.id)}
          strategy={horizontalListSortingStrategy}
        >
          <div className="flex items-center gap-0 overflow-x-auto pb-2 timeline-scroll">
            {clips.map((clip, index) => (
              <div key={clip.id} className="flex items-center">
                <SortableClipCard
                  clip={clip}
                  index={index}
                  onDelete={() => deleteClip(clip.id)}
                  onTrim={(start, end) => setClipTrim(clip.id, start, end)}
                />
                {/* Transition indicator between clips */}
                {index < clips.length - 1 && (
                  <TransitionIcon
                    transition={transitions[index]}
                    onChange={(type) => setTransition(index, type)}
                  />
                )}
              </div>
            ))}
          </div>
        </SortableContext>
      </DndContext>
    </div>
  );
}
