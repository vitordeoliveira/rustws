import React, { useState, useCallback, useEffect } from 'react';
import ReactFlow, {
  Node,
  Edge,
  addEdge,
  Connection,
  useNodesState,
  useEdgesState,
  Controls,
  Background,
  BackgroundVariant,
  NodeTypes,
  EdgeTypes,
  Handle,
  Position,
  MarkerType,
} from 'reactflow';
import dagre from 'dagre';
import 'reactflow/dist/style.css';

// Custom node types for different AWS Step Functions states  
const TaskNode = ({ data, id }: { data: { label: string; resource?: Resource; isEnd?: boolean }; id: string }) => {
  // Get current React Flow instance to check for outgoing edges
  const reactFlowInstance = (window as any).reactFlowInstance;
  const edges = reactFlowInstance?.getEdges?.() || [];
  const hasOutgoingEdges = edges.some((edge: any) => edge.source === id);
  
  // A task is an end state ONLY when explicitly marked as end
  const isEndState = data.isEnd;

  return (
    <div className={`px-4 py-2 shadow-md rounded-md text-white border-2 cursor-pointer relative ${
      isEndState
        ? data.resource 
          ? 'bg-green-600 border-green-700' 
          : 'bg-green-400 border-green-500 border-dashed'
        : data.resource 
          ? 'bg-blue-500 border-blue-600' 
          : 'bg-blue-300 border-blue-400 border-dashed'
    }`}>
      
      {/* Input handle - always present at top */}
      <Handle 
        type="target" 
        position={Position.Top} 
        className="w-3 h-3" 
        style={{ 
          background: '#fb923c', 
          border: '2px solid #ea580c'
        }}
      />
      
      <div className="font-bold flex items-center">
        {data.label}
        {isEndState && (
          <svg className="w-4 h-4 ml-2" fill="currentColor" viewBox="0 0 24 24">
            <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        )}
      </div>
      <div className="text-xs opacity-80">
        {isEndState 
          ? (data.resource ? `End: ${data.resource.resource_name}` : 'End state - Click to assign resource')
          : (data.resource ? `Resource: ${data.resource.resource_name}` : 'Click to assign resource')
        }
      </div>
      
      {/* Output handle - only when NOT an end state */}
      {!isEndState && (
        <Handle 
          type="source" 
          position={Position.Bottom} 
          className="w-3 h-3" 
          style={{ 
            background: '#60a5fa', 
            border: '2px solid #2563eb'
          }}
        />
      )}
    </div>
  );
};


const nodeTypes: NodeTypes = {
  task: TaskNode,
};

interface Resource {
  namespace: string;
  service_type: 'Lambda' | 'Workflow';
  resource_name: string;
}

// Dagre layout configuration
const dagreGraph = new dagre.graphlib.Graph();
dagreGraph.setDefaultEdgeLabel(() => ({}));

const nodeWidth = 200;
const nodeHeight = 80;

// Auto-layout function using dagre
const getLayoutedElements = (nodes: Node[], edges: Edge[], direction = 'TB') => {
  dagreGraph.setGraph({ rankdir: direction, ranksep: 100, nodesep: 50 });

  nodes.forEach((node) => {
    dagreGraph.setNode(node.id, { width: nodeWidth, height: nodeHeight });
  });

  edges.forEach((edge) => {
    dagreGraph.setEdge(edge.source, edge.target);
  });

  dagre.layout(dagreGraph);

  return {
    nodes: nodes.map((node) => {
      const nodeData = dagreGraph.node(node.id);
      return {
        ...node,
        position: {
          x: nodeData.x - nodeWidth / 2,
          y: nodeData.y - nodeHeight / 2,
        },
      };
    }),
    edges,
  };
};

const WorkflowGraph: React.FC = () => {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  
  // Custom edge change handler that also refreshes nodes
  const handleEdgesChange = useCallback((changes: any[]) => {
    onEdgesChange(changes);
    // Force nodes to re-render when edges change
    setNodes((nds) => 
      nds.map((node) => ({ ...node, data: { ...node.data } }))
    );
  }, [onEdgesChange, setNodes]);
  const [workflowDefinition, setWorkflowDefinition] = useState<string>('');
  const [contextMenu, setContextMenu] = useState<{ visible: boolean; x: number; y: number } | null>(null);
  const [nodeContextMenu, setNodeContextMenu] = useState<{ visible: boolean; x: number; y: number; nodeId: string } | null>(null);
  const [edgeContextMenu, setEdgeContextMenu] = useState<{ visible: boolean; x: number; y: number; edgeId: string } | null>(null);
  const [resourceModal, setResourceModal] = useState<{ visible: boolean; nodeId: string } | null>(null);
  const [resources, setResources] = useState<Resource[]>([]);
  const [loadingResources, setLoadingResources] = useState(false);

  // Function to organize layout
  const organizeLayout = useCallback(() => {
    if (nodes.length > 0) {
      const layouted = getLayoutedElements(nodes, edges);
      setNodes(layouted.nodes);
      setEdges(layouted.edges);
    }
  }, [nodes, edges, setNodes, setEdges]);

  const parseWorkflowDefinition = useCallback((definition: string) => {
    if (!definition.trim()) {
      setNodes([]);
      setEdges([]);
      return;
    }

    try {
      const workflow = JSON.parse(definition);
      const states = workflow.States || {};
      const startAt = workflow.StartAt || '';

      const newNodes: Node[] = [];
      const newEdges: Edge[] = [];

      // Create nodes for each state (only Task type supported)
      Object.keys(states).forEach((stateName, index) => {
        const state = states[stateName];
        const stateType = state.Type || 'Task';
        
        // Only support Task type for now
        const nodeType = 'task';

        // Parse resource from JSON if present
        let resource: Resource | undefined = undefined;
        if (state.Resource) {
          // Try to parse our local resource format: namespace:service_type:resource_name
          const localMatch = state.Resource.match(/^([^:]+):([^:]+):(.+)$/);
          
          if (localMatch) {
            resource = {
              namespace: localMatch[1],
              service_type: localMatch[2] as 'Lambda' | 'Workflow',
              resource_name: localMatch[3]
            };
          } else {
            // Fallback: treat as simple resource name
            resource = {
              namespace: 'default',
              service_type: 'Lambda',
              resource_name: state.Resource
            };
          }
        }

        newNodes.push({
          id: stateName,
          type: nodeType,
          position: { x: 0, y: 0 }, // Will be set by layout algorithm
          data: { 
            label: stateName,
            resource: resource,
            isEnd: state.End || false
          },
        });

        // Create edges based on Next (only simple Next transitions for Task nodes)
        if (state.Next) {
          newEdges.push({
            id: `${stateName}-${state.Next}`,
            source: stateName,
            target: state.Next,
            type: 'default',
            markerEnd: {
              type: MarkerType.ArrowClosed,
              width: 20,
              height: 20,
              color: '#374151',
            },
          });
        }
      });

      // Apply auto-layout before setting nodes and edges
      const layouted = getLayoutedElements(newNodes, newEdges);
      setNodes(layouted.nodes);
      setEdges(layouted.edges);
    } catch (error) {
      console.error('Error parsing workflow definition:', error);
      // Keep existing nodes/edges on parse error
    }
  }, [setNodes, setEdges]);

  // Sync with the textarea in the parent form
  useEffect(() => {
    const textarea = document.getElementById('workflowDefinition') as HTMLTextAreaElement;
    if (textarea) {
      const initialDefinition = textarea.value;
      setWorkflowDefinition(initialDefinition);
      
      // Parse initial definition if it exists
      if (initialDefinition && initialDefinition.trim()) {
        parseWorkflowDefinition(initialDefinition);
      }
      
      // Listen for changes in the textarea
      const handleTextareaChange = () => {
        setWorkflowDefinition(textarea.value);
        parseWorkflowDefinition(textarea.value);
      };
      
      textarea.addEventListener('input', handleTextareaChange);
      return () => textarea.removeEventListener('input', handleTextareaChange);
    }
  }, [parseWorkflowDefinition]);

  const onConnect = useCallback(
    (params: Connection) => {
      const newEdge = {
        ...params,
        type: 'default',
        markerEnd: {
          type: MarkerType.ArrowClosed,
          width: 20,
          height: 20,
          color: '#374151',
        },
      };
      setEdges((eds) => addEdge(newEdge, eds));
      // Force nodes to re-render by updating their data slightly
      setNodes((nds) => 
        nds.map((node) => ({ ...node, data: { ...node.data } }))
      );
    },
    [setEdges, setNodes]
  );

  // Handle right-click on pane to show context menu
  const onPaneContextMenu = useCallback((event: React.MouseEvent) => {
    event.preventDefault(); // Prevent browser context menu
    setContextMenu({
      visible: true,
      x: event.clientX,
      y: event.clientY,
    });
    setNodeContextMenu(null); // Hide node context menu
    setEdgeContextMenu(null); // Hide edge context menu
  }, []);

  // Handle right-click on node to show context menu
  const onNodeContextMenu = useCallback((event: React.MouseEvent, node: Node) => {
    event.preventDefault(); // Prevent browser context menu
    setNodeContextMenu({
      visible: true,
      x: event.clientX,
      y: event.clientY,
      nodeId: node.id,
    });
    setContextMenu(null); // Hide pane context menu
    setEdgeContextMenu(null); // Hide edge context menu
  }, []);

  // Handle right-click on edge to show context menu
  const onEdgeContextMenu = useCallback((event: React.MouseEvent, edge: Edge) => {
    event.preventDefault(); // Prevent browser context menu
    setEdgeContextMenu({
      visible: true,
      x: event.clientX,
      y: event.clientY,
      edgeId: edge.id,
    });
    setContextMenu(null); // Hide pane context menu
    setNodeContextMenu(null); // Hide node context menu
  }, []);

  // Handle left-click on task nodes to open resource modal
  const onNodeClick = useCallback((event: React.MouseEvent, node: Node) => {
    if (node.type === 'task') {
      event.stopPropagation();
      setResourceModal({ visible: true, nodeId: node.id });
      fetchResources();
    }
  }, []);

  // Fetch available resources from API
  const fetchResources = useCallback(async () => {
    setLoadingResources(true);
    try {
      const response = await fetch('/api/resources');
      if (response.ok) {
        const data = await response.json();
        setResources(data);
      } else {
        console.error('Failed to fetch resources');
      }
    } catch (error) {
      console.error('Error fetching resources:', error);
    } finally {
      setLoadingResources(false);
    }
  }, []);

  // Assign resource to a task node
  const assignResource = useCallback((nodeId: string, resource: Resource) => {
    setNodes((nds) => 
      nds.map((node) => 
        node.id === nodeId 
          ? { ...node, data: { ...node.data, resource: resource } }
          : node
      )
    );
    setResourceModal(null);
  }, [setNodes]);

  // Delete a node and its connected edges
  const deleteNode = useCallback((nodeId: string) => {
    setNodes((nds) => nds.filter((node) => node.id !== nodeId));
    setEdges((eds) => eds.filter((edge) => edge.source !== nodeId && edge.target !== nodeId));
    setNodeContextMenu(null);
  }, [setNodes, setEdges]);

  // Delete an edge
  const deleteEdge = useCallback((edgeId: string) => {
    setEdges((eds) => eds.filter((edge) => edge.id !== edgeId));
    setEdgeContextMenu(null);
    // Force nodes to re-render to update end state visuals
    setNodes((nds) => 
      nds.map((node) => ({ ...node, data: { ...node.data } }))
    );
  }, [setEdges, setNodes]);

  // Toggle a task as end state
  const toggleTaskEndState = useCallback((nodeId: string) => {
    const node = nodes.find(n => n.id === nodeId);
    const willBeEnd = !node?.data.isEnd;
    
    if (willBeEnd) {
      // Setting as end state - remove all outgoing connections
      setEdges((eds) => eds.filter((edge) => edge.source !== nodeId));
    }
    
    // Toggle the end state flag
    setNodes((nds) => 
      nds.map((node) => 
        node.id === nodeId 
          ? { ...node, data: { ...node.data, isEnd: !node.data.isEnd } }
          : node
      )
    );
    setNodeContextMenu(null);
  }, [nodes, setNodes, setEdges]);

  // Hide context menu when clicking elsewhere
  useEffect(() => {
    const handleClickOutside = () => {
      setContextMenu(null);
      setNodeContextMenu(null);
      setEdgeContextMenu(null);
    };
    
    if (contextMenu?.visible || nodeContextMenu?.visible || edgeContextMenu?.visible) {
      document.addEventListener('click', handleClickOutside);
      return () => document.removeEventListener('click', handleClickOutside);
    }
  }, [contextMenu, nodeContextMenu, edgeContextMenu]);

  // Create a new node at the specified position
  const createNode = useCallback((nodeType: string, position: { x: number; y: number }) => {
    const nodeId = `${nodeType.toLowerCase()}_${Date.now()}`;
    const newNode: Node = {
      id: nodeId,
      type: nodeType,
      position,
      data: { label: nodeId },
    };

    setNodes((nds) => {
      const updatedNodes = [...nds, newNode];
      // Apply auto-layout after adding new node
      const layouted = getLayoutedElements(updatedNodes, edges);
      return layouted.nodes;
    });
    setContextMenu(null);
  }, [setNodes, edges]);

  // Handle context menu item click
  const handleContextMenuAction = useCallback((nodeType: string) => {
    if (contextMenu) {
      // Convert screen coordinates to flow coordinates
      const flowPosition = { x: contextMenu.x - 200, y: contextMenu.y - 100 }; // Approximate offset
      createNode(nodeType, flowPosition);
    }
  }, [contextMenu, createNode]);

  // Update the textarea when nodes/edges change
  const updateWorkflowDefinition = useCallback(() => {
    if (nodes.length === 0) return;

    const states: any = {};
    let startAt = '';

    nodes.forEach((node) => {
      const nodeId = node.id;
      const outgoingEdges = edges.filter(edge => edge.source === nodeId);
      
      if (outgoingEdges.length === 0) {
        // No outgoing edges - Task with no connections
        const stateData: any = {
          Type: 'Task'
        };
        
        // Add resource if assigned
        if (node.data.resource) {
          const resource = node.data.resource;
          // Use our local resource format
          stateData.Resource = `${resource.namespace}:${resource.service_type}:${resource.resource_name}`;
        }
        
        // Add End field ONLY if explicitly marked as end
        if (node.data.isEnd) {
          stateData.End = true;
        }
        
        states[nodeId] = stateData;
      } else if (outgoingEdges.length === 1) {
        // Simple next transition
        const stateData: any = {
          Type: 'Task'
        };
        
        // Add resource if assigned
        if (node.data.resource) {
          const resource = node.data.resource;
          // Use our local resource format
          stateData.Resource = `${resource.namespace}:${resource.service_type}:${resource.resource_name}`;
        }
        
        // Add Next field last
        stateData.Next = outgoingEdges[0].target;
        
        // Add End field if explicitly marked as end (even with connections)
        if (node.data.isEnd) {
          stateData.End = true;
        }
        
        states[nodeId] = stateData;
      }

      // Set first node as start if no startAt is set
      if (!startAt) {
        startAt = nodeId;
      }
    });

    const workflow = {
      Comment: 'Generated workflow',
      StartAt: startAt,
      States: states,
    };

    const jsonString = JSON.stringify(workflow, null, 2);
    setWorkflowDefinition(jsonString);
    
    const textarea = document.getElementById('workflowDefinition') as HTMLTextAreaElement;
    if (textarea) {
      textarea.value = jsonString;
    }
  }, [nodes, edges]);

  // Update textarea when nodes or edges change
  useEffect(() => {
    updateWorkflowDefinition();
  }, [nodes, edges, updateWorkflowDefinition]);

  return (
    <div className="w-full h-full">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={handleEdgesChange}
        onConnect={onConnect}
        onPaneContextMenu={onPaneContextMenu}
        onNodeContextMenu={onNodeContextMenu}
        onEdgeContextMenu={onEdgeContextMenu}
        onNodeClick={onNodeClick}
        nodeTypes={nodeTypes}
        fitView
        attributionPosition="bottom-left"
        onInit={(reactFlowInstance) => {
          (window as any).reactFlowInstance = reactFlowInstance;
        }}
      >
        <Controls />
        <Background variant={BackgroundVariant.Dots} gap={12} size={1} />
        
        {/* Custom Organize Layout Button */}
        <div className="absolute top-4 right-4 z-10">
          <button
            onClick={organizeLayout}
            className="bg-white border border-gray-300 rounded-lg px-3 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-cyan-500 shadow-sm transition-colors duration-200"
            title="Organize Layout"
          >
            <svg className="w-4 h-4 mr-1 inline" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
            </svg>
            Organize
          </button>
        </div>
      </ReactFlow>
      
      {/* Context Menu */}
      {contextMenu?.visible && (
        <div
          className="fixed z-50 bg-white border border-gray-300 rounded-lg shadow-lg py-2 min-w-48"
          style={{
            left: contextMenu.x,
            top: contextMenu.y,
          }}
        >
          <div className="px-3 py-2 text-sm font-medium text-gray-700 border-b border-gray-200">
            Add Node
          </div>
          <button
            className="w-full px-3 py-2 text-left text-sm hover:bg-gray-100 flex items-center"
            onClick={() => handleContextMenuAction('task')}
          >
            <div className="w-3 h-3 bg-blue-500 rounded mr-2"></div>
            Task
          </button>
        </div>
      )}
      
      {/* Node Context Menu */}
      {nodeContextMenu?.visible && (
        <div
          className="fixed z-50 bg-white border border-gray-300 rounded-lg shadow-lg py-2 min-w-40"
          style={{
            left: nodeContextMenu.x,
            top: nodeContextMenu.y,
          }}
        >
          <div className="px-3 py-2 text-sm font-medium text-gray-700 border-b border-gray-200">
            Node Actions
          </div>
          {(() => {
            const node = nodes.find(n => n.id === nodeContextMenu.nodeId);
            
            return (
              <>
                <button
                  className="w-full px-3 py-2 text-left text-sm hover:bg-gray-100 flex items-center text-blue-600"
                  onClick={() => toggleTaskEndState(nodeContextMenu.nodeId)}
                >
                  <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  {node?.data.isEnd ? 'Remove End State' : 'Set as End State'}
                </button>
                <button
                  className="w-full px-3 py-2 text-left text-sm hover:bg-gray-100 flex items-center text-red-600"
                  onClick={() => deleteNode(nodeContextMenu.nodeId)}
                >
                  <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                  Delete Node
                </button>
              </>
            );
          })()}
        </div>
      )}
      
      {/* Edge Context Menu */}
      {edgeContextMenu?.visible && (
        <div
          className="fixed z-50 bg-white border border-gray-300 rounded-lg shadow-lg py-2 min-w-40"
          style={{
            left: edgeContextMenu.x,
            top: edgeContextMenu.y,
          }}
        >
          <div className="px-3 py-2 text-sm font-medium text-gray-700 border-b border-gray-200">
            Connection Actions
          </div>
          <button
            className="w-full px-3 py-2 text-left text-sm hover:bg-gray-100 flex items-center text-red-600"
            onClick={() => deleteEdge(edgeContextMenu.edgeId)}
          >
            <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
            Remove Connection
          </button>
        </div>
      )}
      
      {/* Resource Selection Modal */}
      {resourceModal?.visible && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-lg max-w-2xl w-full mx-4 max-h-96 overflow-hidden">
            <div className="px-6 py-4 border-b border-gray-200">
              <h3 className="text-lg font-medium text-gray-900">
                Select Resource for Task: {resourceModal.nodeId}
              </h3>
            </div>
            
            <div className="px-6 py-4 max-h-64 overflow-y-auto">
              {loadingResources ? (
                <div className="flex items-center justify-center py-8">
                  <div className="text-gray-500">Loading resources...</div>
                </div>
              ) : resources.length === 0 ? (
                <div className="text-center py-8 text-gray-500">
                  No resources available
                </div>
              ) : (
                <div className="space-y-2">
                  {resources.map((resource, index) => (
                    <button
                      key={index}
                      className="w-full p-3 text-left border border-gray-200 rounded-lg hover:bg-gray-50 hover:border-blue-300 transition-colors"
                      onClick={() => assignResource(resourceModal.nodeId, resource)}
                    >
                      <div className="font-medium text-gray-900">
                        {resource.resource_name}
                      </div>
                      <div className="text-sm text-gray-500">
                        {resource.service_type} • {resource.namespace}
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </div>
            
            <div className="px-6 py-4 border-t border-gray-200 flex justify-end">
              <button
                className="px-4 py-2 text-gray-600 hover:text-gray-800 transition-colors"
                onClick={() => setResourceModal(null)}
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default WorkflowGraph;
