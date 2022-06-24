import logging
import yaml
import os

logger = logging.getLogger('Fetching Prices From AMM Pairs Master Logger')

# with open('D:\projects\casino-games-backend\craps\python_files\game-master\config.yml', 'rb') as fp:
#     cfg =  yaml.full_load(fp)

with open("config.yaml", "r") as yamlfile:
    data = yaml.load(yamlfile, Loader=yaml.FullLoader)
    print("Read successful")
print(data)

connectionString = data["connectionString"]
job = data["fetchPrices"]
factories=[]

# def fetchAllFactoriesAddressFromDB():
#     """ Connect to the Database to fetch Factory Address """
#     conn = None
#     try:
#         host=cfg["host"],
#         database=cfg["database"],
#         user=cfg["user"],
#         password=cfg["passwrod"]
#         print('Connecting to the PostgreSQL database...')
#         conn = psycopg2.connect( 
#             host= host,
#             database= database,
#             user= user,
#             password=password
#         )
		
#         # create a cursor
#         cur = conn.cursor()
        
# 	# execute a statement
#         cur.execute('SELECT Id, Address From Factories')

#         # display the PostgreSQL database server version
#         factories = cur.fetchone()
#         print(factories)
       
# 	# close the communication with the PostgreSQL
#         cur.close()
#     except (Exception, psycopg2.DatabaseError) as error:
#         print(error)
#     finally:
#         if conn is not None:
#             conn.close()
#             print('Database connection closed.')

def fetchAllPrices():
    if(job == 'fetchPrices'):
        logger.info("Starting to fetch all prices for all amm_pairs")
        try:
            try:                    
                stream = os.popen("secretd query compute query secret15nr8gcygpn9pq8urkzcf6hvzzvf0s2qtp4ejdj  '{'list_a_m_m_pairs': '{'pagination' : '{'start' : '0', 'limit': 100 }'}'}' --output json")
                output = stream.read()
                print(output)
            except Exception as e:
                logger.error("Encountered error, try to end last game and restarting...")
                # while(True):                                            
        except Exception as e:
            logger.error("critical error, restarting from scratch. " + str(e))
           
fetchAllPrices()      



if __name__ == '__main__':
    fetchAllPrices()