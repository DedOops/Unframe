# Document Internet Client Manifesto

## 1. Project Idea

A modern browser loads more than information.

Along with text, images, and tables, the user receives interfaces, advertisements, recommendations, analytics, background processes, third-party scripts, notifications, attention-retention mechanisms, and application logic that is often unnecessary for obtaining the information itself.

This project starts from a different principle:

> An Internet source provides data. The user's client decides how that data is retrieved, processed, and presented.

We are not building another traditional browser.

We are building a **document Internet client** that extracts information from network sources, converts it into its own controlled document model, and presents it without reproducing the source website's original interface.

---

## 2. Information Before Interface

The primary value of a web page is not its design or implementation.

Its primary value is the information it contains:

- text;
- headings;
- images;
- tables;
- lists;
- code fragments;
- values;
- forms;
- links;
- metadata;
- structured data;
- actions available to the user.

The project does not attempt to reproduce a website's original appearance exactly.

It attempts to determine:

1. what information the source contains;
2. which elements belong to the primary content;
3. which data corresponds to the user's intent;
4. which elements can be removed without losing meaning;
5. which actions are actually required to work with the information.

The source website is treated not as a finished user interface, but as a **data source**.

---

## 3. The Website Does Not Control the User Experience

In a traditional browser, the website largely determines:

- what is loaded;
- which code is executed;
- how the content appears;
- what is placed over the primary content;
- which third-party resources are contacted;
- how much memory and CPU time are consumed;
- which processes continue running after the page is opened;
- which elements distract or retain the user.

In this client, those decisions belong to the user and to the application under the user's control.

A source does not automatically receive permission to:

- execute arbitrary code;
- contact third-party domains;
- load advertising resources;
- start background processes;
- create notifications;
- access the camera, microphone, or other devices;
- store data without functional necessity;
- modify the document after it has been created;
- define the appearance of the client interface.

> A source provides information, but it does not gain control over the user's environment.

---

## 4. A Document Instead of a Running Application

The result of loading a source is not a continuously running web application, but a finite information document.

A document may contain:

- headings;
- text blocks;
- images;
- tables;
- lists;
- quotations;
- code fragments;
- values;
- links;
- forms;
- notifications;
- actions;
- source information.

Once created, a document does not, by default:

- execute code;
- consume CPU in the background;
- send network requests;
- update itself;
- track user activity;
- depend on the source website continuing to run.

An open tab is a stored document, not a continuously running process.

---

## 5. We Do Not Render the Website — We Transform the Source

The project does not have to display the web exactly as the website author designed it.

It transforms an external source into a controlled information document.

```text
Network source
      ↓
Data retrieval
      ↓
Structure and provenance analysis
      ↓
Separation of information from interface noise
      ↓
Conversion into the project's document model
      ↓
Rendering by the client
```

The original page does not need to be rendered.

In the simplest case, the client retrieves HTML, analyzes it as a structured document, and extracts the required content without launching JavaScript, a CSS engine, or a complete browser renderer.

We do not run a website when reading its data is sufficient to obtain the information.

---

## 6. A Unified Information Document Model

All data, regardless of its source or retrieval method, must be converted into one unified information document model.

```text
HTML ────────────────┐
JSON API ────────────┤
RSS and Atom ────────┤
Structured data ─────┤
Local files ─────────┼──→ Information Document Model → Client
Specialized API ─────┤
Application adapter ─┘
```

The client does not display original pages or execute an interface supplied by the source.

The client displays only documents that conform to the project's model.

> Every external source must be converted into our document model before its content is presented to the user.

---

## 7. Model, Serialization, and Rendering Are Separate Layers

The document model must not depend on a particular storage or presentation mechanism.

```text
Document model
      ↓
Serialization
      ↓
Rendering
```

### Document Model

Defines:

- which entities exist;
- which block types are supported;
- how blocks are related;
- which actions are available;
- where the data came from;
- which permissions actions require;
- which state the document is in.

### Serialization

Defines how a document is stored or transferred:

- textual representation;
- binary representation;
- in-memory object;
- local database;
- network protocol.

A specific serialization method must not define the architecture of the project.

### Rendering

The client independently decides:

- which visual components to use;
- how to render text;
- how to display tables;
- how to construct forms;
- how to highlight warnings;
- how to present actions;
- how to adapt a document to different devices.

---

## 8. The Document Describes Meaning, Not Appearance

The internal model must not describe pixels, coordinates, or arbitrary website design.

It must describe semantic entities.

```yaml
document:
  source: https://example.org/article
  title: Article title
  author: Author
  published_at: 2026-06-27

  blocks:
    - type: heading
      level: 1
      text: Article title

    - type: paragraph
      text: The primary content of the article.

    - type: image
      source: https://example.org/image.png
      caption: Image description
      loading: optional

    - type: table
      columns:
        - Name
        - Value
      rows:
        - [...]
```

For an interactive system, a document may also contain actions:

```yaml
document:
  title: Money transfer
  source: https://bank.example

  blocks:
    - type: value
      label: Available balance
      value: 1200
      unit: EUR

    - type: form
      action: transfer
      fields:
        - name: recipient
          type: text
          label: Recipient

        - name: amount
          type: money
          label: Amount

    - type: action
      id: submit-transfer
      label: Transfer
      requires_confirmation: true
```

This is not a copy of the bank's interface.

It is a structured description of information and an allowed action.

---

## 9. Core Layers of the Document Model

The model must include more than visual blocks.

### Content

- text;
- headings;
- images;
- tables;
- lists;
- code;
- values;
- metadata;
- warnings;
- error messages.

### Structure

- sections;
- nesting;
- block order;
- primary and supplementary content;
- relationships between entities;
- logical groups.

### Actions

- open a link;
- submit a form;
- refresh data;
- load the next page;
- download a file;
- confirm an operation;
- cancel an action;
- authenticate;
- retrieve additional data.

### Provenance

- original source;
- URL or API;
- adapter used;
- retrieval time;
- extraction method;
- provenance of an individual block;
- transformations applied.

### Permissions

- network access;
- allowed domains;
- cookies;
- authentication;
- local storage;
- file upload;
- data submission;
- user confirmation.

### State

- loading;
- received;
- stale;
- unavailable;
- error;
- authentication required;
- confirmation required;
- action in progress;
- action completed.

The document model is not merely a markup language.

It is a **protocol between an information source, an adapter, and the user's client**.

---

## 10. The Role of Adapters

An adapter is a translator between an external source and the information document model.

```text
External source
      ↓
    Adapter
      ↓
Information Document Model
      ↓
     Client
```

An adapter explains to the client:

- where the primary information is located;
- which data must be extracted;
- which requests are necessary;
- which elements are noise;
- which actions the source supports;
- how to convert the result into the document model.

Examples:

```text
Article Adapter
Documentation Adapter
Forum Adapter
Product Adapter
Search Adapter
Wikipedia Adapter
GitHub Adapter
Bank Application Adapter
```

An adapter may support:

- a specific website;
- a class of websites;
- a document type;
- a web framework;
- an API;
- a protocol;
- a specialized application.

---

## 11. An Adapter Does Not Create the User Interface

This is one of the project's central architectural rules.

> Adapters do not create user interfaces and do not pass finished pages to the client.

The result of an adapter is always a document that conforms to the project's model.

An adapter must not:

- draw arbitrary windows;
- supply its own interface components;
- control client navigation;
- define fonts, colors, or element placement;
- insert advertising;
- start a persistent background process;
- alter the global behavior of the application.

The client independently determines how each block type and action is presented.

```text
The adapter understands the source.
The model describes information.
The client controls rendering and interaction.
```

---

## 12. Adapter Implementation Does Not Define the Result

An adapter may retrieve data in different ways:

- analyze HTML;
- read structured metadata;
- call a JSON API;
- use RSS or Atom;
- process a specialized protocol;
- interact with an authenticated system;
- use an isolated execution environment;
- apply source-specific rules.

But its output always has the same form:

```text
Source data
      ↓
Standardized information document
```

The client does not need to know how the adapter obtained the information, provided that the adapter:

- respects permissions;
- operates within its budget;
- discloses data provenance;
- does not bypass client policies;
- returns a valid document.

---

## 13. Adapters as Open Support Modules

The project core should not contain knowledge about every existing website and service.

Source support can evolve independently through open modules.

An adapter should be:

- open;
- inspectable;
- isolated from the core;
- permission-constrained;
- replaceable;
- removable;
- independently updatable;
- testable;
- compatible with a versioned document-model contract.

Where possible, an adapter should be declarative: it should describe retrieval and transformation rules rather than execute arbitrary code.

When code execution is necessary, it must occur in a sandbox with explicitly defined permissions.

---

## 14. Open Source and Licensing

The project is intended to be free and open-source software.

The following should be open:

- the client core;
- the information document model;
- the serialization specification;
- the adapter API;
- the permission standard;
- network policies;
- the filtering system;
- built-in adapters;
- information extraction rules;
- content classification mechanisms;
- resource-limiting mechanisms.

Users should be able to inspect:

- which requests the application sends;
- which code is executed;
- where each information block came from;
- which elements were removed;
- which permissions an adapter used;
- which transformations were applied.

Openness is not only a development model, but part of the trust model.

The legal terms for using, modifying, and distributing the project are defined by a separate `LICENSE` file in the repository. This manifesto describes the project's principles and does not replace a software license.

---

## 15. Progressive Data Extraction

The client always begins with the simplest, least expensive, and most controlled method of obtaining information.

The expected progression is:

1. Static HTML.
2. Semantic document elements.
3. Metadata and structured data.
4. Embedded JSON objects.
5. RSS and Atom.
6. Known APIs.
7. Generic content-type adapters.
8. A source-specific adapter.
9. Limited isolated processing.
10. A full renderer only as an explicit last-resort fallback.

Each next level is used only when the previous level cannot obtain sufficient content.

---

## 16. The Information Firewall

The client acts as an information firewall between the user and the source.

It controls not only network requests, but also the composition of the information presented.

The client may classify elements as:

- primary material;
- original source;
- reference information;
- navigation;
- related material;
- advertising;
- affiliate link;
- social block;
- algorithmic recommendation;
- authentication element;
- system message;
- required action.

The user may define a policy:

```text
Show:
✓ primary material
✓ original sources
✓ links within the text
✓ tables
✓ document images
✓ required actions

Hide:
✗ advertising
✗ affiliate links
✗ social buttons
✗ algorithmic recommendations
✗ unrelated navigation
✗ attention-retention mechanisms
```

The project's goal is not merely to block specific advertising domains.

Its goal is to prevent the user from being automatically enrolled in mechanisms unrelated to the user's original intent.

---

## 17. Explicit Network Activity

A network request must not be considered acceptable merely because the source website wants to make it.

Every request must have a clear purpose:

- primary document;
- required resource;
- image;
- data API;
- user action;
- authentication;
- optional third-party resource;
- prohibited resource.

By default, the client aims for the following model:

- first-party data is allowed within a defined budget;
- third-party scripts are blocked;
- advertising and analytics requests are blocked;
- images may be loaded separately or on demand;
- background connections are blocked;
- persistent connections require separate permission;
- automatic data transmission is minimized.

---

## 18. No Ambient Engagement

The project does not claim to provide absolute anonymity.

Its goal is to eliminate unnecessary background participation in other parties' systems.

By default, a document should not:

- send analytics;
- load advertising;
- maintain an active session without necessity;
- refresh recommendations;
- preserve tracking identifiers;
- execute background timers;
- load new elements without user action;
- maintain an infinite feed;
- transmit behavioral information.

> The user interacts only with the information and actions they explicitly requested.

---

## 19. Resource Contract

Every document and adapter operates within a limited budget.

A budget may include:

- maximum downloaded data;
- number of network requests;
- allowed domains;
- memory usage;
- CPU time;
- processing time;
- maximum image resolution;
- data retention period;
- cookie access;
- permission for background activity.

```text
Document:

HTML and data: up to 500 KB
Images: up to 3 MB
Third-party domains: blocked
JavaScript: blocked
Background requests: blocked
Processing time: up to 2 seconds
```

Resource limits are part of the architecture, not an optional optimization.

---

## 20. Privacy Without Absolute Promises

Using the Internet necessarily requires network interaction.

The project does not promise magical anonymity or complete invisibility.

Instead, it follows verifiable principles:

- do not transmit data without functional necessity;
- do not perform hidden telemetry;
- do not store identifiers without a clear reason;
- do not contact third-party services automatically;
- expose network activity to the user;
- separate public and private content;
- process private data locally by default;
- explicitly disclose remote processing.

We do not promise complete isolation from the network.

We promise the absence of implicit and unnecessary participation in other parties' systems.

---

## 21. Local Processing by Default

The project's default mode assumes local processing.

The following should be processed locally:

- authenticated pages;
- banking applications;
- email;
- workplace systems;
- medical services;
- personal accounts;
- sources containing sensitive data.

Remote processing may be used only for public content and only under a transparent policy.

Server-side processing must not become a hidden mandatory dependency of the client.

---

## 22. Adapter Boundaries

Open modularity must not undermine the project's core principles.

An adapter must not silently:

- disable network policy;
- exceed its assigned budget;
- access cookies belonging to other sources;
- execute unrestricted code;
- start background processes;
- add advertising;
- send telemetry;
- modify the client core;
- control the interface;
- hide its own activity.

When an adapter needs additional privileges, the user must receive a concrete request.

```text
This adapter requires:

— access to bank.example.com;
— storage of first-party cookies;
— execution of an isolated authentication flow;
— submission of POST requests;
— up to 256 MB of memory during the session.

Third-party domains:
— captcha.example.net.

Background activity:
— blocked.
```

Even an adapter for a complex banking application must operate under an explicitly described contract.

---

## 23. Actions Must Be Transparent

The client must not receive a button with unknown behavior.

Every action must include a description:

```yaml
action:
  id: submit-transfer
  adapter: bank-example
  target: https://bank.example/api/transfer
  method: POST

  requires:
    - authenticated-session
    - explicit-confirmation
```

The client must understand:

- which adapter created the action;
- which source it belongs to;
- which data will be submitted;
- which permissions are required;
- whether confirmation is required;
- whether the operation can be canceled.

---

## 24. Data Provenance Must Be Visible

Each information block should preserve its provenance whenever possible.

```text
Title          ← HTML <h1>
Author         ← structured data
Date           ← page metadata
Primary text   ← HTML <article>
Price          ← JSON API
Image          ← first-party resource
```

Each document may include an audit record:

```text
Source: example.org
Retrieved: June 27, 2026, 16:40
Network requests: 3
Data received: 214 KB
Requests blocked: 28
Third-party domains: 0
Scripts executed: 0
Adapter used: Generic Article Adapter
```

The user should be able to understand how the document was constructed.

---

## 25. Multiple Clients, One Model

A unified document model allows different clients to be built:

```text
Desktop client
Mobile client
Terminal client
Screen reader client
E-ink client
Accessibility client
```

All of them receive the same information document and present it differently.

This separates source understanding from a particular interface or device.

---

## 26. Document Caching and Storage

Because the result is a finite document, it can be stored.

The client may preserve:

- document structure;
- block provenance;
- retrieval time;
- adapter used;
- action state;
- reading position;
- allowed resources.

Reopening a document does not always require rerunning the adapter or contacting the source.

A document can exist independently of a running web application.

---

## 27. Adapter Testability

An adapter must be testable independently of the client interface.

```text
Source input
      ↓
    Adapter
      ↓
Expected information document
```

For example, a product adapter should return:

- product name;
- price;
- specifications;
- availability;
- images;
- supported actions;
- no advertising blocks.

This allows adapters to be tested automatically and their behavior compared between versions.

---

## 28. Not a Universal Browser

The project does not have to replace Chrome, Firefox, or Safari in every scenario.

It does not have to support:

- complex web editors;
- browser games;
- WebGL applications;
- video editing;
- arbitrary SaaS applications;
- exact reproduction of every interface;
- every website without adaptation.

Rejecting universality is a deliberate architectural decision.

The project's primary use case is:

> Obtain maximum informational value with the minimum amount of third-party interface, executable code, and background activity.

---

## 29. Failure Is Better Than a Hidden Fallback

If the client cannot safely and correctly extract information, it must report that limitation.

It must not silently:

- launch a full browser;
- execute all of the website's JavaScript;
- send the page to a third-party server;
- allow additional domains;
- bypass the defined resource budget;
- abandon its own document model.

```text
The content could not be obtained in document mode.

Reason:
the source is generated dynamically and does not expose accessible data.

Available actions:
— use an installed adapter;
— temporarily allow isolated processing;
— open the source in an external browser.
```

Controlled incompatibility is better than silently abandoning the project's principles.

---

## 30. Core Project Structure

### Core

Responsible for:

- navigation;
- document lifecycle;
- settings;
- permissions;
- tabs;
- history;
- state persistence.

### Network Gateway

Responsible for:

- HTTP;
- TLS;
- DNS;
- cookies;
- redirects;
- limits;
- filtering;
- network activity logging.

### Extraction Engine

Responsible for:

- HTML parsing;
- primary content extraction;
- metadata processing;
- embedded data processing;
- API discovery;
- element classification.

### Adapter Runtime

Responsible for:

- adapter loading;
- permission checks;
- sandboxing;
- declarative rules;
- versioning;
- document-model compatibility.

### Information Document Model

Defines:

- block types;
- document structure;
- actions;
- data provenance;
- permissions;
- states;
- serialization.

### Native Renderer

Responsible for rendering:

- text;
- images;
- tables;
- code;
- forms;
- links;
- notifications;
- different document types.

### Policy Engine

Defines:

- network restrictions;
- resource budgets;
- permissions;
- privacy rules;
- allowed fallback mechanisms.

### Audit View

Displays:

- network requests;
- blocked resources;
- adapters used;
- block provenance;
- resource consumption;
- permissions applied;
- actions performed.

---

## 31. Minimum First Version

The first version must not attempt to solve the entire Internet.

It must prove the core ideas.

The minimum version supports:

- opening a URL;
- retrieving static HTML;
- extracting primary text;
- headings;
- paragraphs;
- lists;
- links;
- simple tables;
- images on demand;
- script blocking;
- third-party resource blocking;
- the project's own document model;
- its own native renderer;
- a network request log;
- one generic adapter;
- support for one external adapter;
- storage of the resulting document.

The first version is successful if it can open a set of information-oriented websites without a full browser engine and present them as documents in its own model.

---

## 32. Criteria for Staying True to the Idea

Before adding a new feature, ask:

1. Does it help obtain information?
2. Does it require executing someone else's interface?
3. Does it create background activity?
4. Can it be implemented as a controlled adapter?
5. Does the adapter return a standard document?
6. Is it clear which data and resources are used?
7. Does it violate local processing by default?
8. Does it increase universality at the cost of control?
9. Can it be constrained by an explicit contract?
10. Does it allow the source to control the client interface?

If a feature turns the project back into a conventional browser, it should be moved into an adapter, isolated fallback, or not implemented.

---

## 33. Final Architectural Principles

```text
The source provides data.

The adapter understands the source.

The model describes information and actions.

The client controls rendering.

Policies constrain network and resources.

The user remains in control.
```

---

## 34. Final Project Principle

> We do not display the Internet the way websites want to present it.

> We extract information from the Internet and present it the way the user has chosen.

The project does not fight the modern web by attempting to build one more faster browser.

It proposes a different model:

```text
Do not run the website.
Do not reproduce the website.
Do not give the website control over the interface.

Retrieve the data.
Verify its provenance.
Convert it into our own model.
Create the document.
Present the information.
```
