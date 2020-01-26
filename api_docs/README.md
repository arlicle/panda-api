# Auth api doc


the test_data proviede the front end request and response test case data

#### request post /login/

body: 
```
{username:"edison", password:"123"}
```
get response:
```
{code:-1, msg:"password incorrect"}
```

#### request post /login/

body: 
```
{username:"lily", password:"123"}
```
get response:
```
{code:-2, msg:"username not exist"}
```


#### request post /login/

body: 
```
{username:"root", password:"123"}
```
get response:
```
{code:1, msg:"login success", token:"fjdlkfjlafjdlaj3jk2l4j"}
```





#### request post /login/

body: 
```
{username:"lily"}
```
get response:
```
{code:1, msg:"login success", token:"fjdlkfjlafjdlaj3jk2l4j"}
```


#### request post /login/

body: 
```
{password:"123"}
```
get response:
```
{code:-1, msg:"username is required"}
```


#### request get /logout/

query: 
```
{id:1, username:"root"}
```
get response:
```
{code:1, msg:"logout success"}
```
 
 
#### request get /logout/

query: empty

get response:
```
{code:-1, msg:"error"}
```



 
#### request get /logout/

query: 
```
{id:3, username:"lily"}
```
get response:
```
{code:-1, msg:"username and id not match"}
```


## a object list

``` json5
result: 
    [{
        id:{name:"article id", type:"number"},
        title:{name:"article title"},
        category_name:{name:"article category name"},
        author_name:{name:"article author name"},
        tag:{name:"article tag"},
        created:{name:"article created time", type:"number"}
    }]

```