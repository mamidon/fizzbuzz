# fizzbuzz

A note to any reviewers on implementation: I didn't get around to adding real error handling for situations such as:
1. Dispute, resolve, or chargeback transactions that cross accounts 
2. Malformed or invalid transactions
3. Extreme precision issues

Exactly what the proper error handling is depends greatly on the exact use case -- do we trust our partner systems? are we exposed to the public?
I did start to add logic to attempt to associate partially formed transactions with their accounts, which would be useful for staff operating such a system, 
but decided against proceeding further due to time.

I also added some dead code warning suppressions due to time, but generally I would prefer to not do that.

I would also prefer not to store all transactions in memory, and offload as much of that to a database as possible. 
