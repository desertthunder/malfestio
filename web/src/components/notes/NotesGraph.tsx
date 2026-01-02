import type { Note } from "$lib/model";
import { extractWikilinkTitles, resolveWikilink } from "$lib/wikilink";
import { useNavigate } from "@solidjs/router";
import { drag } from "d3-drag";
import type { Simulation, SimulationLinkDatum, SimulationNodeDatum } from "d3-force";
import { forceCenter, forceCollide, forceLink, forceManyBody, forceSimulation } from "d3-force";
import { select } from "d3-selection";
import { zoom, type ZoomBehavior, zoomIdentity } from "d3-zoom";
import type { Component } from "solid-js";
import { createEffect, createMemo, onCleanup, onMount } from "solid-js";

export type GraphNode = SimulationNodeDatum & { id: string; title: string; tags: string[]; linkCount: number };

export type GraphLink = SimulationLinkDatum<GraphNode> & { source: string | GraphNode; target: string | GraphNode };

type NotesGraphProps = {
  notes: Note[];
  currentNoteId?: string;
  onNodeClick?: (noteId: string) => void;
  width?: number;
  height?: number;
};

const NODE_RADIUS = 8;
const LINK_DISTANCE = 100;
const CHARGE_STRENGTH = -300;

export const NotesGraph: Component<NotesGraphProps> = (props) => {
  const navigate = useNavigate();
  let containerRef: HTMLDivElement | undefined;
  let simulation: Simulation<GraphNode, GraphLink> | undefined;
  let zoomBehavior: ZoomBehavior<SVGSVGElement, unknown> | undefined;

  const width = () => props.width ?? 800;
  const height = () => props.height ?? 600;

  const graphData = createMemo(() => {
    const notes = props.notes;
    const nodeMap = new Map<string, GraphNode>();
    const links: GraphLink[] = [];

    for (const note of notes) {
      nodeMap.set(note.id, { id: note.id, title: note.title, tags: note.tags, linkCount: 0 });
    }

    for (const note of notes) {
      const titles = extractWikilinkTitles(note.body);
      for (const title of titles) {
        const targetNote = resolveWikilink(title, notes);
        if (targetNote && targetNote.id !== note.id) {
          const sourceNode = nodeMap.get(note.id);
          const targetNode = nodeMap.get(targetNote.id);
          if (sourceNode && targetNode) {
            sourceNode.linkCount++;
            targetNode.linkCount++;
            links.push({ source: note.id, target: targetNote.id });
          }
        }
      }
    }

    return { nodes: Array.from(nodeMap.values()), links };
  });

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const handleNodeClick = (_: any, d: GraphNode) => {
    if (props.onNodeClick) {
      props.onNodeClick(d.id);
    } else {
      navigate(`/notes/${d.id}`);
    }
  };

  const computeColor = (d: GraphNode) =>
    d.id === props.currentNoteId
      ? "fill-blue-500"
      : d.linkCount > 0
      ? "fill-emerald-500"
      : "fill-slate-400 dark:fill-slate-500";
  const initializeGraph = () => {
    if (!containerRef) return;

    const svg = select(containerRef).select<SVGSVGElement>("svg").attr("viewBox", `0 0 ${width()} ${height()}`);
    const g = svg.select<SVGGElement>(".graph-container");
    const data = graphData();

    zoomBehavior = zoom<SVGSVGElement, unknown>().scaleExtent([0.1, 4]).on("zoom", (event) => {
      g.attr("transform", event.transform);
    });

    svg.call(zoomBehavior);
    svg.call(zoomBehavior.transform, zoomIdentity.translate(width() / 2, height() / 2));

    simulation = forceSimulation<GraphNode, GraphLink>(data.nodes).force(
      "link",
      forceLink<GraphNode, GraphLink>(data.links).id((d) => d.id).distance(LINK_DISTANCE),
    ).force("charge", forceManyBody().strength(CHARGE_STRENGTH)).force("center", forceCenter(0, 0)).force(
      "collision",
      forceCollide().radius(NODE_RADIUS * 2),
    );

    const linkSelection = g.select<SVGGElement>(".links").selectAll<SVGLineElement, GraphLink>("line").data(
      data.links,
      (d) => `${(d.source as GraphNode).id ?? d.source}-${(d.target as GraphNode).id ?? d.target}`,
    ).join("line").attr("class", "stroke-slate-600 dark:stroke-slate-500").attr("stroke-opacity", 0.6).attr(
      "stroke-width",
      1.5,
    );

    const nodeSelection = g.select<SVGGElement>(".nodes").selectAll<SVGGElement, GraphNode>(".node").data(
      data.nodes,
      (d) => d.id,
    ).join("g").attr("class", "node cursor-pointer").on("click", handleNodeClick);

    nodeSelection.selectAll("circle").data((d) => [d]).join("circle").attr(
      "r",
      (d) => NODE_RADIUS + Math.min(d.linkCount, 5),
    ).attr("class", computeColor).attr("stroke", "var(--color-slate-900)").attr("stroke-width", 2);

    nodeSelection.selectAll("text").data((d) => [d]).join("text").text((d) =>
      d.title.length > 20 ? d.title.slice(0, 20) + "..." : d.title
    ).attr("x", 14).attr("y", 4).attr(
      "class",
      "text-xs fill-slate-300 dark:fill-slate-400 pointer-events-none select-none",
    );

    const dragBehavior = drag<SVGGElement, GraphNode>().on("start", (event, d) => {
      if (!event.active) simulation?.alphaTarget(0.3).restart();
      d.fx = d.x;
      d.fy = d.y;
    }).on("drag", (event, d) => {
      d.fx = event.x;
      d.fy = event.y;
    }).on("end", (event, d) => {
      if (!event.active) simulation?.alphaTarget(0);
      d.fx = null;
      d.fy = null;
    });

    nodeSelection.call(dragBehavior);

    simulation.on("tick", () => {
      linkSelection.attr("x1", (d) => (d.source as GraphNode).x ?? 0).attr("y1", (d) => (d.source as GraphNode).y ?? 0)
        .attr("x2", (d) => (d.target as GraphNode).x ?? 0).attr("y2", (d) => (d.target as GraphNode).y ?? 0);

      nodeSelection.attr("transform", (d) => `translate(${d.x ?? 0}, ${d.y ?? 0})`);
    });
  };

  onMount(() => {
    initializeGraph();
  });

  createEffect(() => {
    graphData();
    if (simulation) {
      simulation.stop();
    }
    initializeGraph();
  });

  onCleanup(() => {
    simulation?.stop();
  });

  return (
    <div
      ref={containerRef}
      class="w-full h-full bg-slate-900/50 rounded-xl border border-slate-700 overflow-hidden"
      data-testid="notes-graph">
      <svg class="w-full h-full" style={{ "min-height": `${height()}px` }}>
        <g class="graph-container">
          <g class="links" />
          <g class="nodes" />
        </g>
      </svg>
    </div>
  );
};
