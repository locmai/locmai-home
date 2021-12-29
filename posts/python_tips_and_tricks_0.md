---
title: 'Python tips and tricks you may not know'
description: "Helpful tips and tricks for scripting/programming in Python"
date: 2021-09-10
author:
  name: Loc Mai
tags:
  - python
published: true
layout: layouts/post.njk
---

I will just throw some random tips and tricks in Python that I've found that I thought was really helpful but most people don't use much.

For my own convenience, all of the examples below is written on Python 3.9.6 cuz I have it on my machine rigth now.

## String formatting: Please use the fking f-string!

I'm still seeing a lot of these string-formatted code in the old-school way like these:

``` python/
name = "Loc Mai"
job  = "SRE"
print("Hello, my name is {fname} and I'm and {fjob}".format(fname="Loc Mai", fjob="SRE"))
```

OR

``` python/
name = "Loc Mai"
job  = "SRE"
print("Hello, my name is {0} and I'm and {1}".format(name, job))
```

The code above will print out `"Hello, my name is Loc Mai and I'm an SRE"` 


Here is the shorter one with f-string that print out the same output:

``` python/
name = "Loc Mai"
job  = "SRE"
print(f"Hello, my name is {name} and I'm and {job}")
```

Clean, DRY, easier to read for human's eyes.

## Decorator pattern
    
This is a pattern where you simply write decorator functions those could wrap around the other functions and change some of their behaivours.

Let's say I want to calculate the time execution of a function when it ran:

```python/
from datetime import datetime

def timer(func):
    def wrapper():
        start = datetime.now()
        func()
        end = datetime.now()
        print(f"Time execution for `{func.__name__}()`: {end - start}")
    return wrapper

def hello_world():
    print("Hello World!")

hello_world = timer(hello_world)
```

Now if we called the hello_world() function:

```
>>> hello_world()
Hello World!
Time execution for hello_world: 0:00:00.000038
```

So the code above could be written like this with the @timer decorator before the hello_world function and also add another function called `bye_world()`:

```python/
from datetime import datetime
import time

def timer(func):
    def wrapper():
        start = datetime.now()
        func()
        end = datetime.now()
        print(f"Time execution for `{func.__name__}()`: {end - start}")
    return wrapper

@timer
def hello_world():
    print("Hello World!")

@timer
def bye_world():
    print("Bye bye World!")
    time.sleep(2) # Sleep for 2 seconds after saying good bye
```

Now try the bye_world():

```
>>> bye_world()
Bye bye World!
Time execution for bye_world: 0:00:02.003713
```

More complicated use case like adding an ability to use arguments in the decorator could solve by wrapping the function like this:


```python/
from datetime import datetime
import time

def timer(threshold):
    def wrap(func):
        def wrapped_f(*args):
            start = datetime.now()
            func()
            end = datetime.now()
            print(f"Time execution for `{func.__name__}()`: {end - start} vs ")
            print(f"Execution took: {(end - start).total_seconds()}")
            print(f"Threshold: {threshold}")
        return wrapped_f
    return wrap

@timer(1)
def hello_world():
    print("Hello World!")
```

## List comprehension

I use this a lot to create a new list based on existing lists with shorter syntax provided, let's say you have a list of dict about pokemon's profile:

```python/
pokemon_list = [
  {
    'name': 'Charizard',
    'id': 6,
    'type': ['Fire', 'Flying']
  },
  {
    'name': 'Cinderace',
    'id': 815,
    'type': 'Fire'
  },
  {
    'name': 'Pikachu',
    'id': 25,
    'type': 'Electric'
  },
]
```

If I wanna filter and get the list of 'Fire' pokemon, I could do with just one line:

```python/
fire_pokemon_name_list = [pokemon['name'] for pokemon in pokemon_list if 'Fire' in pokemon['type']]
print(fire_pokemon_name_list)

# result: ['Charizard', 'Cinderace']
```

## Type hinting

Python go with dynamic typing model which is good if you didn't really care about type-safe. But if you were writing library code that the others would use, you could provide some typing hints for your users:

For example, we have library.py with greeting:

```python/
# library.py
def greeting(name: str) -> str:
    return f'Hello {name}'
```

Pretending we are the users importing the library code:

```
from library import greeting
```

From the IDE, we could now see what parameter needed with what type, and the type of the returned value from the function greeting()

## Bonus: Postional parameters and keyword parameters

Never thought I should put this on the list until I have to debate with someone that using this properly can make their code better and they refused to use it, then later on see the useful of it.

Python provides a way to explicit define which parameters must be defined postionally, which must be defined by keywords. 

Simple example is:

```python/
def simple_function(a, b, /, c, d, *, e, f):
    print(a, b, c, d, e, f)
```

The following calls will result in:

```python/
simple_function(10, 20, 30, d=40, e=50, f=60) # valid call, this will print out all the parameters
```

Explain: The syntax `/` to indicate that some function parameters must be specified positionally and cannot be used as keyword arguments, and the `*` indicate the parameters after it must be keywords.

So the following calls will be invalid to call:

```python/
simple_function(10, b=20, c=30, d=40, e=50, f=60)   # b cannot be a keyword argument
simple_function(10, 20, 30, 40, 50, f=60)           # e must be a keyword argument
```

Moving on with that, if we were unsure about the number of arguments to pass in the functions, we could use *args and **kwargs instead.

By that, you could add any number of arguments into the function dynamically, for example we have the following `adder` which allow to add any positional parameters:

```python/
def adder(*num):
    sum = 0
    
    for n in num:
        sum = sum + n

    print("Sum of all:",sum)

adder(3,5)
adder(4,5,6,7)
adder(1,2,3,5,6)
```

The case I had the debate was how to write a custom Graph helper that extends and return the basic [Graph class](https://maibaloc.com/posts/vault_argocd_gitops) 

```python/
import grafanalib.core as Graph

def CustomGraph(data_source, title, expressions, **kwargs) -> Graph:
    prefix='custom'
    return Graph(
        title=f"{prefix}-{title}",
        dataSource=data_source,
        targets=targets,
        **kwargs
    )
```

.
.
.
.
Well, that's it for now, happy coding.