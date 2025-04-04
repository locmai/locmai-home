# Reflections on Infrastructure as Code

After several years of working as a Site Reliability Engineer (SRE) on a medium-to-large infrastructure codebase, and after countless moments of frustration over past implementations, designs, or even just poorly worded documentation, I've developed some strong opinions on the subject. Some of these insights are ideas I wish I had adopted earlier, and others are lessons worth sharing.

Disclaimer: This is not a criticism of anyone else’s mistakes (except my own). Rather, it is a self-reflection on what has worked, what hasn’t, and what could have been done differently. If you find yourself relating to these experiences, perhaps we are not so different after all, we suck, but we suck together.

# Test the code

To be completely honest, I haven't always written as many tests for my infrastructure code as I should have. I wrote just enough to claim that there were “some safety measures” in place—basic checks that allowed me to say, “See? There are tests!” When questioned by managers or reviewers, I often responded with, “Oh, this is temporary,” or “I’ll add them later.” Sometimes, the urgency of business needs pushed testing to the bottom of my priority list. But the truth is, my lack of testing often stemmed from my own skill issues rather than time constraints.

You can usually distinguish a good engineer from a bad one by evaluating their code quality, including the presence and robustness of tests. We all acknowledge that testing is crucial, yet we rarely treat it as such.

I understand that not all SREs come from a software engineering background—many transition from system administration or networking roles. The syntax of .rego or .go files look intimidating to me, as well. But if we are already writing Terraform’s HCL, which has its own quirks, why not push ourselves a little further and embrace testing tools like Open Policy Agent (OPA) or Terratest? In the long run, a well-tested infrastructure saves time and prevents headaches.

## The Case for 'var.global'

Let’s consider the use of a global data provider for a Terraform codebase.

Using a schema-validated+battle-tested global variable such as var.global can be beneficial—it acts as a contract between users and maintainers. Suppose var.global contains the network topology of your infrastructure deployment. In that case, every virtual network must adhere to a well-defined structure, ensuring consistency in how resources are created and used. This prevents errors on both ends: users cannot provide incorrect inputs, and maintainers cannot introduce breaking changes without proper handling.

Without a clear contract between users and maintainers, chaos ensues. Imagine you are using a Terraform module maintained by someone careless—someone like past me. You are given a module with a variable called foo, which you use to create a virtual machine. The next day, I decide to rename foo to bar without proper documentation or warnings. Suddenly, your code breaks, and you’re left scrambling to figure out why. Now, extend this scenario to multiple deployments, all failing due to this arbitrary change.

Conversely, imagine I’m the user and your module requires a virtual network name to remain constant from the initial deployment. Let’s say the network was originally named "spit-fire", but I rename it to "swallow-water" without considering the consequences. Suddenly, all pipelines break, and you, as the maintainer, are blamed—despite making no changes to your module.

How do we prevent such issues? First, by writing tests. Second, by implementing schema validation to enforce well-structured inputs. And if a structural change is truly necessary, versioning and pre-deployment scripts can help manage transitions smoothly.

I've been on both sides of this problem, and I’ve been the cause of these issues more times than I’d like to admit.

## Know Your End Users

One of my earlier debates revolved around whether a Terraform module should be opinionated (strict with limited flexibility) or highly configurable. Initially, I leaned toward opinionated modules, assuming that users would be knowledgeable enough to use them correctly. However, we failed to ask the most important question: Who are our users?

Understanding your end users—whether they are developers, infrastructure engineers, or platform teams—shapes how you design and document your modules. Striking the right balance between flexibility and usability ensures that modules serve their intended purpose without unnecessary complexity.


## Still

Infrastructure as Code is an evolving practice, and mistakes are inevitable. However, reflecting on past missteps and continuously improving our approach is what makes us better engineers. Testing, clear contracts between users and maintainers, and a deep understanding of end users can go a long way in making infrastructure code more maintainable and resilient.


