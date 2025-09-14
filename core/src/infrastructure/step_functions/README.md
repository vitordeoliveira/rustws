# Step Functions Infrastructure

This directory contains the infrastructure for managing and executing step function workflows in RUSTWS Core.

## 🎯 **RUSTWS Philosophy**

**"Lambdas define contracts, workflows connect them, transformations are lambdas too."**

### **Key Principles:**

- ✅ **Clean Lambda Contracts** - Each lambda defines exact input/output
- ✅ **Simple Workflows** - Just connect lambdas, no complex transformations
- ✅ **Transformation Lambdas** - Need data transformation? Create a lambda for it
- ✅ **No Workflow Complexity** - Avoid Parameters, InputPath, OutputPath, ResultPath
- ✅ **Testable Components** - Every piece of logic is in a testable lambda

**Result: Clean, maintainable, debuggable step functions! 🎉**

## Directory Structure

```
step_functions/
├── README.md           # This file
├── mod.rs              # StepFunctionStorage struct
└── workflows/          # JSON workflow definitions
    └── hello_world.json # Example chained workflow
```

## Workflow JSON Format

Step functions are defined using the Amazon States Language (ASL) JSON format. Here are the most important fields and concepts:

### 1. Basic Structure

```json
{
  "Comment": "Description of what this workflow does",
  "StartAt": "FirstStateName",
  "States": {
    "StateName": {
      "Type": "Task|Choice|Wait|Succeed|Fail"
      // State-specific configuration
    }
  }
}
```

### 2. Key Fields Reference

#### **StartAt**

- **Purpose**: Defines which state begins the workflow execution
- **Type**: String (must match a state name in `States`)
- **Example**: `"StartAt": "FirstHelloWorld"`

#### **States**

- **Purpose**: Contains all the states/steps in your workflow
- **Type**: Object with state names as keys
- **Required**: Every workflow must have at least one state

### 3. Task State Fields

#### **Type**

- **Purpose**: Defines the type of state
- **Values**: `"Task"` (lambda execution), `"Choice"` (conditional), `"Wait"` (delay), `"Succeed"` (end successfully), `"Fail"` (end with error)
- **Example**: `"Type": "Task"`

#### **Resource**

- **Purpose**: Specifies which function/service to execute
- **RUSTWS Format**: `"rustws:lambda:function_name"`
- **AWS Format**: `"arn:aws:lambda:region:account:function:name"`
- **Example**: `"Resource": "rustws:lambda:hello_world"`

#### **Next** vs **End**

- **Next**: Move to another state → `"Next": "NextStateName"`
- **End**: Terminate workflow → `"End": true`
- **Rule**: Every state must have either `Next` or `End`

### 4. Clean Data Passing Architecture (RUSTWS Approach)

**RUSTWS Philosophy: Lambdas define their own contracts, workflows just connect them.**

#### **Core Principle**

- ✅ **Lambda functions define their exact input/output structures**
- ✅ **Workflows pass data directly between lambdas without transformation**
- ✅ **No complex Parameters, InputPath, OutputPath manipulation**
- ✅ **Clean, maintainable, and predictable data flow**

#### **Why This Approach?**

1. **Type Safety** - Each lambda knows exactly what it expects
2. **Testability** - Lambdas can be tested independently with known inputs
3. **Maintainability** - No complex data transformation logic in workflows
4. **Clarity** - Easy to understand what data flows between functions
5. **Reusability** - Lambdas with clear contracts can be used in multiple workflows

#### **Simple Task Definition**

```json
{
  "Type": "Task",
  "Resource": "rustws:lambda:hello_world",
  "Next": "NextState"
}
```

**That's it!** No InputPath, OutputPath, Parameters, or ResultPath needed.

#### **Data Flow Example**

**Input to workflow:**

```json
{ "userId": "12345", "name": "John" }
```

**FirstLambda receives exactly:**

```json
{ "userId": "12345", "name": "John" }
```

**FirstLambda outputs:**

```json
{
  "processedUserId": "12345",
  "greeting": "Hello John",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

**SecondLambda receives exactly:**

```json
{
  "processedUserId": "12345",
  "greeting": "Hello John",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

**No transformation, no data manipulation - just clean handoffs.**

### 5. Data Transformation - The RUSTWS Way

**Need to transform data between lambdas? Create a transformation lambda!**

#### **✅ RUSTWS Approach: Transformation Lambdas**

Instead of complex workflow transformations, create dedicated lambdas:

```json
{
  "StartAt": "GetUserData",
  "States": {
    "GetUserData": {
      "Type": "Task",
      "Resource": "rustws:lambda:get_user_data",
      "Next": "TransformForProcessing"
    },
    "TransformForProcessing": {
      "Type": "Task",
      "Resource": "rustws:lambda:transform_user_data",
      "Next": "ProcessUser"
    },
    "ProcessUser": {
      "Type": "Task",
      "Resource": "rustws:lambda:process_user",
      "End": true
    }
  }
}
```

#### **Why Transformation Lambdas?**

1. **✅ Testable** - Unit test transformation logic independently
2. **✅ Reusable** - Use same transformation in multiple workflows
3. **✅ Maintainable** - All transformation logic in one place
4. **✅ Type Safe** - Clear input/output contracts
5. **✅ Debuggable** - Easy to trace and debug transformations

#### **Example Transformation Lambda:**

```rust
// transform_user_data.rs
pub struct TransformUserInput {
    pub user_id: String,
    pub raw_data: serde_json::Value,
}

pub struct TransformUserOutput {
    pub processed_user_id: String,
    pub formatted_data: UserData,
    pub metadata: TransformMetadata,
}
```

#### **❌ Avoid Workflow Transformations**

These fields exist but should be avoided in RUSTWS:

- **`InputPath`** - Create a transformation lambda instead
- **`OutputPath`** - Create a transformation lambda instead
- **`ResultPath`** - Create a transformation lambda instead
- **`Parameters`** - Create a transformation lambda instead

**Keep workflows simple - move complexity into lambdas!**

### 6. JSON Path Syntax

| Syntax         | Description          | Example                            |
| -------------- | -------------------- | ---------------------------------- |
| `$`            | Root of current data | `"InputPath": "$"`                 |
| `$.field`      | Access object field  | `"userId.$": "$.user.id"`          |
| `$.array[0]`   | Access array element | `"first.$": "$.items[0]"`          |
| `$.array[0:5]` | Array slice          | `"subset.$": "$.data[0:5]"`        |
| `$$`           | Context object       | `"time.$": "$$.State.EnteredTime"` |

### 7. Choice States (Conditional Logic)

```json
{
  "Type": "Choice",
  "Choices": [
    {
      "Variable": "$.status",
      "StringEquals": "success",
      "Next": "SuccessState"
    },
    {
      "Variable": "$.count",
      "NumericGreaterThan": 10,
      "Next": "HighCountState"
    }
  ],
  "Default": "DefaultState"
}
```

### 8. Error Handling

#### **Retry**

```json
"Retry": [
  {
    "ErrorEquals": ["States.TaskFailed"],
    "IntervalSeconds": 2,
    "MaxAttempts": 3,
    "BackoffRate": 2.0
  }
]
```

#### **Catch**

```json
"Catch": [
  {
    "ErrorEquals": ["States.TaskFailed"],
    "Next": "ErrorHandlerState",
    "ResultPath": "$.error"
  }
]
```

## RUSTWS Resource Format

For local lambda functions, use the RUSTWS resource format:

```json
"Resource": "rustws:lambda:function_name"
```

This references:

- `function_name.rs` in `src/infrastructure/lambdas/source/`
- `function_name.wasm` in `src/infrastructure/lambdas/wasm/`

## Common Patterns

### 1. **Sequential Processing**

```json
{
  "StartAt": "Step1",
  "States": {
    "Step1": { "Next": "Step2" },
    "Step2": { "Next": "Step3" },
    "Step3": { "End": true }
  }
}
```

### 2. **Conditional Branching**

```json
{
  "StartAt": "CheckCondition",
  "States": {
    "CheckCondition": {
      "Type": "Choice",
      "Choices": [
        {
          "Variable": "$.isValid",
          "BooleanEquals": true,
          "Next": "ProcessData"
        }
      ],
      "Default": "HandleError"
    }
  }
}
```

### 3. **Data Transformation Pattern**

**❌ Old Complex Approach:**

```json
{
  "Type": "Task",
  "Resource": "rustws:lambda:process_user",
  "ResultPath": "$.processResult",
  "Parameters": {
    "userId.$": "$.user.id",
    "userName.$": "$.user.profile.name",
    "timestamp.$": "$$.State.EnteredTime"
  }
}
```

**✅ RUSTWS Clean Approach:**

```json
{
  "GetUser": {
    "Type": "Task",
    "Resource": "rustws:lambda:get_user",
    "Next": "TransformUserData"
  },
  "TransformUserData": {
    "Type": "Task",
    "Resource": "rustws:lambda:transform_user_for_processing",
    "Next": "ProcessUser"
  },
  "ProcessUser": {
    "Type": "Task",
    "Resource": "rustws:lambda:process_user",
    "End": true
  }
}
```

**Benefits:**

- **Testable transformation logic** in dedicated lambda
- **Reusable transformations** across workflows
- **Clear separation of concerns**
- **Type-safe data contracts**

## Best Practices (RUSTWS Approach)

### 1. **Lambda Design First**

- ✅ **Define clear input/output contracts** for each lambda
- ✅ **Use TypeScript/Rust types** to define lambda interfaces
- ✅ **Test lambdas independently** with known input/output
- ✅ **Document lambda contracts** in code comments

### 2. **Workflow Design**

- ✅ **Keep workflows simple** - just connect lambdas
- ✅ **Use descriptive state names** - `ValidateUser` not `Task1`
- ✅ **Avoid data transformation** in workflows
- ✅ **Create transformation lambdas** instead of using Parameters/InputPath/OutputPath
- ✅ **Let lambdas handle their own data needs**

### 3. **Error Handling**

- ✅ **Add `Retry` and `Catch` blocks** for robust workflows
- ✅ **Let lambdas return structured error responses**
- ✅ **Use Choice states** for conditional error handling

### 4. **Documentation**

- ✅ **Document lambda contracts** clearly
- ✅ **Add workflow comments** explaining business logic
- ✅ **Keep README files** for complex workflows

### 5. **Testing Strategy**

- ✅ **Unit test lambdas** with mock input/output
- ✅ **Integration test workflows** with real lambdas
- ✅ **Validate workflow JSON** structure
- ✅ **Test error scenarios** and edge cases

## Testing Workflows

To test a workflow:

1. **Validate JSON** - Ensure proper JSON syntax
2. **Check state references** - All `Next` states must exist
3. **Verify resource paths** - Lambda functions must exist
4. **Test data flow** - Verify InputPath/OutputPath logic
5. **Handle errors** - Test failure scenarios

## Example Files

- `hello_world.json` - Demonstrates clean lambda chaining with well-defined contracts
- More examples coming soon...

## Lambda Contract Examples

### **Example 1: Processing Lambda**

**Input:**

```rust
pub struct HelloWorldInput {
    // Define what this lambda expects
    pub message: Option<String>,
    pub context: Option<serde_json::Value>,
}
```

**Output:**

```rust
pub struct HelloWorldOutput {
    // Define what this lambda produces
    pub greeting: String,
    pub timestamp: String,
    pub processed: bool,
}
```

### **Example 2: Transformation Lambda**

**Input:**

```rust
pub struct TransformUserInput {
    pub user_id: String,
    pub raw_profile: serde_json::Value,
    pub preferences: Vec<String>,
}
```

**Output:**

```rust
pub struct TransformUserOutput {
    pub processed_user_id: String,
    pub formatted_profile: UserProfile,
    pub normalized_preferences: Vec<UserPreference>,
    pub transformation_metadata: TransformMetadata,
}
```

### **Clean Workflow Connection:**

```json
{
  "GetUserData": {
    "Type": "Task",
    "Resource": "rustws:lambda:get_user_data",
    "Next": "TransformUser"
  },
  "TransformUser": {
    "Type": "Task",
    "Resource": "rustws:lambda:transform_user_data",
    "Next": "ProcessUser"
  },
  "ProcessUser": {
    "Type": "Task",
    "Resource": "rustws:lambda:hello_world",
    "End": true
  }
}
```

**Benefits:**

- ✅ **Each lambda has a single responsibility**
- ✅ **Transformation logic is isolated and testable**
- ✅ **Clean data contracts between all steps**
- ✅ **Reusable transformation lambdas**
- ✅ **No complex workflow syntax**

Clean, simple, maintainable! 🎉
