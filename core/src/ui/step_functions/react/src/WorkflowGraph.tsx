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
} from 'reactflow';
import 'reactflow/dist/style.css';

// Custom node types for different AWS Step Functions states
const TaskNode = ({ data }: { data: { label: string; resource?: Resource } }) => (
  <div className={`px-4 py-2 shadow-md rounded-md text-white border-2 cursor-pointer ${
    data.resource 
      ? 'bg-blue-500 border-blue-600' 
      : 'bg-blue-300 border-blue-400 border-dashed'
  }`}>
    <div className="font-bold">{data.label}</div>
    <div className="text-xs opacity-80">
      {data.resource ? `Resource: ${data.resource.resource_name}` : 'Click to assign resource'}
    </div>
  </div>
);

const ChoiceNode = ({ data }: { data: { label: string } }) => (
  <div className="px-4 py-2 shadow-md rounded-md bg-yellow-500 text-white border-2 border-yellow-600">
    <div className="font-bold">{data.label}</div>
    <div className="text-xs opacity-80">Choice</div>
  </div>
);

const WaitNode = ({ data }: { data: { label: string } }) => (
  <div className="px-4 py-2 shadow-md rounded-md bg-purple-500 text-white border-2 border-purple-600">
    <div className="font-bold">{data.label}</div>
    <div className="text-xs opacity-80">Wait</div>
  </div>
);

const SucceedNode = ({ data }: { data: { label: string } }) => (
  <div className="px-4 py-2 shadow-md rounded-md bg-green-500 text-white border-2 border-green-600">
    <div className="font-bold">{data.label}</div>
    <div className="text-xs opacity-80">Succeed</div>
  </div>
);

const FailNode = ({ data }: { data: { label: string } }) => (
  <div className="px-4 py-2 shadow-md rounded-md bg-red-500 text-white border-2 border-red-600">
    <div className="font-bold">{data.label}</div>
    <div className="text-xs opacity-80">Fail</div>
  </div>
);

const StartNode = ({ data }: { data: { label: string } }) => (
  <div className="px-4 py-2 shadow-md rounded-md bg-emerald-500 text-white border-2 border-emerald-600">
    <div className="font-bold">{data.label}</div>
    <div className="text-xs opacity-80">Start</div>
  </div>
);

const nodeTypes: NodeTypes = {
  task: TaskNode,
  choice: ChoiceNode,
  wait: WaitNode,
  succeed: SucceedNode,
  fail: FailNode,
  start: StartNode,
};

interface Resource {
  namespace: string;
  service_type: 'Lambda' | 'Workflow';
  resource_name: string;
}

const WorkflowGraph: React.FC = () => {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const [workflowDefinition, setWorkflowDefinition] = useState<string>('');
  const [contextMenu, setContextMenu] = useState<{ visible: boolean; x: number; y: number } | null>(null);
  const [nodeContextMenu, setNodeContextMenu] = useState<{ visible: boolean; x: number; y: number; nodeId: string } | null>(null);
  const [resourceModal, setResourceModal] = useState<{ visible: boolean; nodeId: string } | null>(null);
  const [resources, setResources] = useState<Resource[]>([]);
  const [loadingResources, setLoadingResources] = useState(false);

  // Sync with the textarea in the parent form
  useEffect(() => {
    const textarea = document.getElementById('workflowDefinition') as HTMLTextAreaElement;
    if (textarea) {
      setWorkflowDefinition(textarea.value);
      
      // Listen for changes in the textarea
      const handleTextareaChange = () => {
        setWorkflowDefinition(textarea.value);
        parseWorkflowDefinition(textarea.value);
      };
      
      textarea.addEventListener('input', handleTextareaChange);
      return () => textarea.removeEventListener('input', handleTextareaChange);
    }
  }, []);

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

      // Create nodes for each state
      Object.keys(states).forEach((stateName, index) => {
        const state = states[stateName];
        const stateType = state.Type || 'Task';
        const isStart = stateName === startAt;
        
        let nodeType = 'task';
        if (stateType === 'Choice') nodeType = 'choice';
        else if (stateType === 'Wait') nodeType = 'wait';
        else if (stateType === 'Succeed') nodeType = 'succeed';
        else if (stateType === 'Fail') nodeType = 'fail';
        else if (isStart) nodeType = 'start';

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
          position: { x: 100 + (index % 3) * 200, y: 100 + Math.floor(index / 3) * 150 },
          data: { 
            label: stateName,
            resource: resource
          },
        });

        // Create edges based on Next, Choices, and Default
        if (state.Next) {
          newEdges.push({
            id: `${stateName}-${state.Next}`,
            source: stateName,
            target: state.Next,
            label: 'Next',
            type: 'smoothstep',
          });
        }

        if (state.Choices) {
          state.Choices.forEach((choice: any, choiceIndex: number) => {
            if (choice.Next) {
              newEdges.push({
                id: `${stateName}-choice-${choiceIndex}`,
                source: stateName,
                target: choice.Next,
                label: 'Choice',
                type: 'smoothstep',
              });
            }
          });
        }

        if (state.Default) {
          newEdges.push({
            id: `${stateName}-default`,
            source: stateName,
            target: state.Default,
            label: 'Default',
            type: 'smoothstep',
          });
        }
      });

      setNodes(newNodes);
      setEdges(newEdges);
    } catch (error) {
      console.error('Error parsing workflow definition:', error);
      // Keep existing nodes/edges on parse error
    }
  }, [setNodes, setEdges]);

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
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

  // Hide context menu when clicking elsewhere
  useEffect(() => {
    const handleClickOutside = () => {
      setContextMenu(null);
      setNodeContextMenu(null);
    };
    
    if (contextMenu?.visible || nodeContextMenu?.visible) {
      document.addEventListener('click', handleClickOutside);
      return () => document.removeEventListener('click', handleClickOutside);
    }
  }, [contextMenu, nodeContextMenu]);

  // Create a new node at the specified position
  const createNode = useCallback((nodeType: string, position: { x: number; y: number }) => {
    const nodeId = `${nodeType.toLowerCase()}_${Date.now()}`;
    const newNode: Node = {
      id: nodeId,
      type: nodeType,
      position,
      data: { label: nodeId },
    };

    setNodes((nds) => [...nds, newNode]);
    setContextMenu(null);
  }, [setNodes]);

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
        // End state - determine type based on node type
        let stateData: any = {};
        
        if (node.type === 'task') {
          // Task nodes are always Type: "Task"
          stateData = {
            Type: 'Task'
          };
          
          // Add resource if assigned
          if (node.data.resource) {
            const resource = node.data.resource;
            // Use our local resource format
            stateData.Resource = `${resource.namespace}:${resource.service_type}:${resource.resource_name}`;
          }
        } else {
          // Other node types default to Succeed for end states
          stateData = { Type: 'Succeed' };
        }
        
        states[nodeId] = stateData;
      } else if (outgoingEdges.length === 1) {
        // Simple next transition
        const stateData: any = {
          Type: node.type === 'task' ? 'Task' : 'Pass',
          Next: outgoingEdges[0].target,
        };
        
        // Add resource if assigned (only for task nodes)
        if (node.type === 'task' && node.data.resource) {
          const resource = node.data.resource;
          // Use our local resource format
          stateData.Resource = `${resource.namespace}:${resource.service_type}:${resource.resource_name}`;
        }
        
        states[nodeId] = stateData;
      } else {
        // Choice state
        const choices = outgoingEdges
          .filter(edge => edge.label !== 'Default')
          .map(edge => ({
            Variable: '$.status',
            StringEquals: 'success',
            Next: edge.target,
          }));
        
        const defaultEdge = outgoingEdges.find(edge => edge.label === 'Default');
        
        states[nodeId] = {
          Type: 'Choice',
          Choices: choices,
          ...(defaultEdge && { Default: defaultEdge.target }),
        };
      }

      // Find start node
      if (node.type === 'start' || !startAt) {
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
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onPaneContextMenu={onPaneContextMenu}
        onNodeContextMenu={onNodeContextMenu}
        onNodeClick={onNodeClick}
        nodeTypes={nodeTypes}
        fitView
        attributionPosition="bottom-left"
      >
        <Controls />
        <Background variant={BackgroundVariant.Dots} gap={12} size={1} />
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
          {/* <button
            className="w-full px-3 py-2 text-left text-sm hover:bg-gray-100 flex items-center"
            onClick={() => handleContextMenuAction('start')}
          >
            <div className="w-3 h-3 bg-emerald-500 rounded mr-2"></div>
            Start
          </button> */}
          <button
            className="w-full px-3 py-2 text-left text-sm hover:bg-gray-100 flex items-center"
            onClick={() => handleContextMenuAction('task')}
          >
            <div className="w-3 h-3 bg-blue-500 rounded mr-2"></div>
            Task
          </button>
          {/* <button
            className="w-full px-3 py-2 text-left text-sm hover:bg-gray-100 flex items-center"
            onClick={() => handleContextMenuAction('choice')}
          >
            <div className="w-3 h-3 bg-yellow-500 rounded mr-2"></div>
            Choice
          </button>
          <button
            className="w-full px-3 py-2 text-left text-sm hover:bg-gray-100 flex items-center"
            onClick={() => handleContextMenuAction('wait')}
          >
            <div className="w-3 h-3 bg-purple-500 rounded mr-2"></div>
            Wait
          </button>
          <button
            className="w-full px-3 py-2 text-left text-sm hover:bg-gray-100 flex items-center"
            onClick={() => handleContextMenuAction('succeed')}
          >
            <div className="w-3 h-3 bg-green-500 rounded mr-2"></div>
            Succeed
          </button>
          <button
            className="w-full px-3 py-2 text-left text-sm hover:bg-gray-100 flex items-center"
            onClick={() => handleContextMenuAction('fail')}
          >
            <div className="w-3 h-3 bg-red-500 rounded mr-2"></div>
            Fail
          </button> */}
        </div>
      )}
      
      {/* Node Context Menu */}
      {nodeContextMenu?.visible && (
        <div
          className="fixed z-50 bg-white border border-gray-300 rounded-lg shadow-lg py-2 min-w-32"
          style={{
            left: nodeContextMenu.x,
            top: nodeContextMenu.y,
          }}
        >
          <div className="px-3 py-2 text-sm font-medium text-gray-700 border-b border-gray-200">
            Node Actions
          </div>
          <button
            className="w-full px-3 py-2 text-left text-sm hover:bg-gray-100 flex items-center text-red-600"
            onClick={() => deleteNode(nodeContextMenu.nodeId)}
          >
            <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
            Delete Node
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
