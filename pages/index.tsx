import Head from 'next/head'

export default function Home() {
    return (
        <div className="container">
            <Head>
                <title>Create Next App</title>
                <link rel="icon" href="/favicon.ico" />
            </Head>

            <main>
                <form method="post" className="flex justify-center pt-20">
                    <div>
                        <h1 className="mb-6 text-5xl">🚀 AskBid: A Search Exchange 🌚</h1>

                        <div className="flex border border-gray-200 rounded p-4 shadow text-xl">
                            <div>🔎</div>
                            <input type="text" className="w-full outline-none px-3" name="query" required />
                            <div>🇺🇸</div>
                        </div>

                        <div className="mt-8 text-center">
                            <button
                                className="mr-3 bg-green-200 border border-green-300 py-3 px-4 rounded hover:bg-green-400 hover:border-green-500">
                                Ask the internet
                            </button>
                        </div>
                    </div>
                </form>
            </main>
        </div>
    )
}
