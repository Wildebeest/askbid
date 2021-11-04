# Introduction

**AskBid** is a decentralized search market on the Solana blockchain. It utilizes a novel price discovery mechanism to find the best answer for a user's query.

Here to build an information trading bot? Check out our walkthrough!

Here to build an alternate frontend? Check out our walkthrough!

Confused? Read on...

---

## Remind me, how do search engines work?

Traditional search engines like Google or Bing have four large responsibilities.
1. Crawl the internet for useful content.
2. Serve a website that can receive search queries from users.
3. Efficiently respond with useful content for each search query.
4. Mix in advertisements with the content to make money.

The internet has organized itself around this model of information retrieval for the past 20 years.

## What's the problem with this?
This model puts the search engine in conflict with the rest of the internet.
* Users want their answers as quickly as possible. Search engines have a strong incentive to extract useful content from the web and show it directly to the user. This means that the search engine gets paid rather than the actual content producer.
* As an ads-based business, search engines must choose between serving their financial interest or serving their users. Over time, search engines find the maximum amount of ad-load they can place in front of the relevant content they've extracted from the internet.
* As a central gatekeeper, there is a strong temptation for search engines to use their position to shape the content produced on the internet. For example, Google used their central position to push a technology called AMP so that websites would stop using advertising providers other than Google.

These market forces are very strong. It means that search users today get the maximum amount of ads they can tolerate, followed by editorialized results from a shrinking internet that is unsupported by revenue.

## Ok, so how does a search exchange work?
Users need an efficient way to find answers that are most relevant to them. A search exchange realigns the incentives so that the most profitable result is also the most relevant result.

1. A **user** submits a query via a **frontend** to the exchange, along with some SOL. This repo contains a Web2 frontend for submitting queries where the user pays the listing fee from a wallet they control. In the long run, hopefully there will be multiple frontends.
2. **Information traders** are notified of a query by Solana and list results they think are relevant on the exchange. Each result opens a limited-time prediction market around the question "Will the user who submitted the query mark this result as most relevant?"
3. **Information traders** can mint token pairs against a result by escrowing some SOL with the exchange. A "yes" token can be redeemed for escrowed SOL if the user decides that this result is the most relevant. A "no" token can be redeemed for escrowed SOL if the user chooses a different result, or if the user does not choose a result before the market expires.
4. **Information traders** then trade these tokens on the exchange. These tokens derive their value from the underlying SOL, and their price will converge to the probability that the user will decide that this result is the most relevant.
5. The **frontend** observes the current market prices of the tokens for each result and uses them to rank the results. This repo contains a simple frontend that just sorts by descending price.
6. The **user** decides which result is most relevant and submits a decision to the exchange via the **frontend**. The exchange then refunds the user a redemption amount of SOL with their query.

## How does that address the problems with traditional search engines?
Each participant in the search market must find it more profitable to return relevant user results than to engage in alternate activities.
* The **frontend** should find it more profitable to serve relevant results than ads. This is accomplished by granting the frontend a cut of the fees charged to information traders.
* **Content producers** can list their results directly without waiting for bots to crawl their sites, and sell "no" tokens to get compensated for the value they are providing.
* **Information traders** can get started with ranking areas they are familiar with rather than needing to build a good general search engine to get any traffic. 
* Since the firehose of queries, results, and decisions is public on the Solana blockchain, **information traders** with novel ranking ideas can test them more easily.



