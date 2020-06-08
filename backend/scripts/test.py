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
        put_movie(request: 
            { 
                title: \"The Irishman\",
                imdb_id: \"tt1302006\",
                published_at: \"2019-09-27T00:00:00Z\",
                roles: [
                    {
                        actor_last_name: \"Deniro\",
                        actor_first_name: \"Robert\",
                        character_names: [\"Frank Sheeran\"],
                    },
                    {
                        actor_last_name: \"Pacino\",
                        actor_first_name: \"Al\",
                        character_names: [\"Jimmy Hoffa\"],
                    }
                ]
            } 
        ) 
    }"""
})
pprint(response)

response = invoke({
    "query": """mutation 
    { 
        put_movie(request: 
            { 
                title: \"21 Jump Street\",
                imdb_id: \"tt1232829\",
                published_at: \"2012-03-16T00:00:00Z\",
                roles: [
                    {
                        actor_last_name: \"Tatum\",
                        actor_first_name: \"Channing\",
                        character_names: [\"Jenko\"],
                    },
                    {
                        actor_last_name: \"Hill\",
                        actor_first_name: \"Jonah\",
                        character_names: [\"Schmidt\"],
                    },
                    {
                        actor_last_name: \"Larson\",
                        actor_first_name: \"Brie\",
                        character_names: [\"Molly Tracey\"],
                    }
                ]
            } 
        ) 
    }"""
})
pprint(response)

response = invoke({
    "query": """mutation 
    { 
        put_movie(request: 
            { 
                title: \"Heat\",
                imdb_id: \"tt0113277\",
                published_at: \"1995-12-15T00:00:00Z\",
                roles: [
                    {
                        actor_last_name: \"Deniro\",
                        actor_first_name: \"Robert\",
                        character_names: [
                            \"Neil McCauley\"
                        ],
                    },
                    {
                        actor_last_name: \"Pacino\",
                        actor_first_name: \"Al\",
                        character_names: [
                            \"Lt. Vincent Hanna\"
                        ],
                    }
                ]
            } 
        ) 
    }"""
})
pprint(response)

response = invoke({
    "query": """query 
    {
        get_movie(title: \"21 Jump Street\", published: 2012) { 
            meta { 
                title, 
                published_at 
            }, 
            roles { 
                actor { 
                    last_name, 
                    first_name 
                }, 
                characters { 
                    name 
                } 
            } 
        }
    }"""
})
pprint(response)
