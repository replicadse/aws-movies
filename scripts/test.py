import boto3
from pprint import pprint
import json

DEFAULT_CLIENT = boto3.client('lambda')
DEFAULT_FUNCTION = "movies-handler-grapqhl"

def invoke(payload, *, client = DEFAULT_CLIENT, function = DEFAULT_FUNCTION):
    if payload is not None:
        response = client.invoke(
            FunctionName=function,
            InvocationType='RequestResponse',
            Payload=json.dumps(payload))
    else:
        response = client.invoke(
            FunctionName=function,
            InvocationType='RequestResponse')
    return json.loads(json.loads(response['Payload'].read()))

response = invoke({
    "query": """mutation 
    { 
        post_movie(request: 
            { 
                title: \"21 Jump Street\",
                watched: \"2020-01-01T12:00:00.0000Z\",
                actors: [
                    "Channing Tatum",
                    "Jonah Mills"
                ]
            } 
        ) 
    }"""
})
movie_id = response["data"]["post_movie"]
pprint(response)

response = invoke({
    "query": "query { get_movie(id: \"" + movie_id + "\") { id, title, watched, actors } }"
})
pprint(response)

response = invoke({
    "query": "query { list_movies }"
})
pprint(response)

response = invoke({
    "query": "mutation { delete_movie(id: \"" + movie_id + "\") }"
})
pprint(response)
