## Introduction
In an ideal world, given rules and a clear goal, we'd formulate precise utility functions to gauge outcomes. However, our flawed ability to estimate long-term outcomes not only affects our everyday decisions but also hinders the analytical capabilities of scientists and engineers. To bridge these gaps, experts have traditionally turned to computational tools, like Monte Carlo simulations and formal methods. These tools amplify human reasoning, uncovering patterns and insights that might otherwise remain obscured.

Unlike traditional methods that operate in isolation, PARX orchestrates these diverse tools in a unified platform designed to augment users mental model of the underlying system By seamlessly integrating system description with algorithmic search, PARX facilitates deeper understanding for researchers.

This document will spotlight PARX's efficacy in program search and delve into its prospective roles in various commercial and research arenas.
##  A Search Engine for *Something*
Drafting system requirements and UML modeling are vital preliminary steps for software design.  For complex protocols and mission-critical components, a formal specification is also developed to validate the high-level-correctness of system behavior
![[Untitled Diagram.drawio (8).png]]

Yet, capturing emergent behaviors—central to our current project—poses a challenge. For our sake, emergence here means "something you didn't know was true but you know that you care about". This definition has a couple of consequences. 
1. we have to be able to operate under the premise these behaviors are always initially under-specified.
2. these behaviors occur across abstraction levels. Initially this will take the form of bridging high level requirements to a vector consisting of watchpoints tracking low level states of what is being examined. 
3. once we've captured sufficient instances of this kind of behavior, how do we universally instantiate them and come up with a new *Emergent Specification* for generating new instances
#### Terminology:
*executable*: any complex system that is semi-deterministic and whose runtime behavior can be observed. The thing we are interested in understanding
*specification*: anything from a formal specification to a natural language description of high level invariants or SLA's of a system
*event stream*: in order to deal with variable response time and to ensure ordering for the agent's sequential exploration, we use an intermediate abstraction of an event stream 
*agent*: RL agent described below exploring the executable to find violations of invariants in the specification

### Existing Approaches 
Interactions with **PARX** start with configuration an by writing a natural language description of how the the *executable* is intended to behave. The user waits for a length of time and the trained agent is deployed and produces output. The user examines and re-ranks the output and creates a new specification reflecting both their prior and expanded beliefs. This process repeats until the end-result (accumulated across iterations) is satisfactory. 

Underneath the hood, there are two functional components. One, a handler that takes the belief descriptions provided by the end user and extracts that which can be formally specified to a formal intermediate representation. The rest is provided as instructions as part of the human feedback process. \footnote{this process will start as a series of decision rules and will be refined in the future} Second, a grey-box heuristic-driven program search is performed. 

We steer clear from exhaustive, coverage-based searches and do not provide any formal assurances about the programs being examined. Instead, we zero in on issues found in stateful, multi-tenant codebases with exacting SLA's (availability, performance, data persistence and security) and identifying salient compositional patterns of user behavior at build-time. We assume our system will be used alongside fuzzers and formally generated parsers. These tools are stateless and a good start to the stateful exploration we plan on performing. 

The race towards maximizing a coverage-based, stateless strategy in fuzzing is counterproductive from a commercial standpoint. Trading computational power for _number_ of vulnerabilities isn't interesting. Many existing fuzzers neglect  the rich set of beliefs companies have about correctness and relevance yielding false positives and failing to capture software runtime dynamics. 

### Implementation
Starting with a  specification consisting of a mix of  natural language, input-output examples, and program sketches and an executable that reads and writes inputs from an event stream. We produce an emergent description capturing surprising and relevant behavior triggered by long sequences of actions and infeasibility criteria provided by the user.
![[Screenshot 2023-09-27 at 1.47.36 PM.png]]
## Implementation
The formal model of **PARX** consists of  $P$ as the program or system under test, $Z$ a specification of one or more watchpoints of $P$ defined by initial global state $z_0 = init(z-1)$ and a set of transition rules $z_{n+1}  = P(z_n)$, Q an actor/critic agent \cite{konda1999actor} which learns  a policy $\pi(a_t | s_t, \theta)$ that learns to select actions that maximize how the probability of discovering behaviors of $S$ not present in $Z$ or explicit counter-examples $z_n$ reached from $z_0$ using only legal transition rules. $Z$ is an intermediate representation constructed from the free form description of $P$. 

$Q$ is implemented with a policy gradient algorithm. Policy gradient algorithms estimate return using samples of episodes to update policy parameters to converge on an  estimate of the sum over state distribution and the action value function parameterized by the policy $J_\theta$ . (#TODO; add an aside about generalized advantage estimation) The critic, a simple neural network, predicts the value of a given action (general advantage estimation). In order to predict the overall value of the trajectory, the equivalent prediction of the next state is used, This single-step value  estimate of the actor $\pi_\theta(a|s)$ acts as a baseline. In fact, single-step value function is the optimal baseline for reducing variance of the policy without introducing biasThe policy collects this information over a sequence of epochs. Each epoch the reward is computed (as configured in the environment) and regularized by the probability ratio $r(\theta) = \frac{\pi_\theta(a|s)}{\pi_{\theta_{old}}(a|s)}$ computed between  old and new policies . Any changes to the policy are also clipped, minimizing the degree to which the policy can change one epoch over the other relative to the reward incentivizing training stability.  We only want to allow for substantial change in the policy if there is a correspondingly substantial increase in how interesting the behaviors we are seeing. 
**Risk Aware Policy Gradient Methods** Conventionally, policy gradient methods are evaluated in terms of both their training and test accuracy. We are interested in an approach that can tolerate limited over-fitting and whose emphasis is on searching for extreme states versus average performance. In order to achieve this we use use a risk-aware policy-gradient method  $J_{risk}(\theta; \epsilon)$  which only uses traces above the $(1 - \epsilon)$ for updating the policy parameters \cite{petersen2019deep}
##### Action Space
The policy samples concrete actions and writes them to the event stream buffering the underlying system. This and an atomic read form a single transaction or environment step. The action space is specified as a series of interesting inputs, high-level function calls, and restricted ranges of inputs similar to seeds provided to a fuzzing application. Optionally, users can use fuzzers to select interesting inputs. This fuzzing process is not back-propagated as part of the overall RL loop and suggests inputs based off of stateless coverage metrics. Future directions for this process could include queries to a Large Language Model fine-tuned on a relevant corpus similar to how counter-examples are generated in Everparse
##### State Space
An ideal embedding strategy takes multi-modal vectors and finds a continuous representation that  which pushes semantically similar instances together and pulls dissimilar instances apart. Currently, the embedding strategies we use are very simple and do not include structural awareness of the underlying system.  We partially observe the program through a few watchpoints.
##### Reward 
We will elicit the specification of external reward criteria when initializing the program. However, we assume this external reward criteria is sparse and include two mechanisms for negotiating this. 
###### Intrinsic Reward
First, we use an intrinsic reward taken from  "Curiosity Exploration for Self-Supervised Prediction" \cite{pathak2017curiosity}. This includes an  **Inverse Model**which embeds states $s_t$ and $s_{t+1}$ into features  $\phi(s_t)$ and $\phi(s_{t+ 1})$ using the surrogate task of predicting $\hat{a_t}$. Then the **Forward Dynamics Model**  takes $a^{s}_t$ or an action embedding along with the $\phi(s_t)$ to predict $\phi(\hat{s}_{t 1})$. Its loss  $L( \hat{\phi(s_{t+ 1})}, \phi(s_{t+ 1}))$  motivates the exploration of surprising, actionable states. In our program search use case, the states are watchpoints placed in the Linux Kernel with GDB. 
###### Reinforcement Learning: Human Feedback
Our curiosity based reward extracts and ranks state machines at the end of a given iteration. The user is asked to retain only  the state machines that are relevant to their use case (in part using their own provided criteria that was formally unenforceable). This fine tunes a reward model that provides an additional reward source. We have not yet performed detailed experimentation about what the ideal mechanism for annotating traces is to recover the user's preference.
###### Specification Based Reward
If $Z$ exists, explicit violations observed by reaching a state that contains an infeasible property can be given a high reward. It is assumed that this reward is very sparse.
## Engineering Roadmap 
Our current approach has been deployed as part of the HARDEN program whose goal is to model emergent execution in a manner that  minimizes transportability costs across different computer programs. **PARX** has been evaluated as a "powerful approach to systematically locate composable chains or primitives to achieve a desired vulnerability goal" by Northrop Grumman. 

Our goals for the end of 2023 are to support the following features
1. An ecosystem that supports multiple use cases that read and write to our event stream abstraction
2. shift to natural language based specification
	1. create a preliminary routing system for natural language and for constructing an IR of a formal specification

work with us  ecosystem github / email us / join our community 


  