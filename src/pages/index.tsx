import type { NextPage } from "next"
import { useRouter } from "next/router"
import { useEffect } from "react"

import AppHead from "@/components/AppHead"

const Home: NextPage = () => {
  /*
  // Tauri Invoke Example
  const [buttonDesc, setButtonDesc] = useState<string>(
    "Waiting to be clicked. This calls 'on_button_clicked' from Rust.",
  )
  const onButtonClick = () => {
    invoke<string>("on_button_clicked")
      .then((value) => {
        setButtonDesc(value)
      })
      .catch(() => setButtonDesc("Failed to invoke Rust command 'on_button_clicked'"))
  }

  useGlobalShortcut("CommandOrControl+P", () => {
    console.log("Ctrl+P was pressed!")
  })
  */

  // redirect to preferences page
  const router = useRouter()
  useEffect(() => {
    void router.replace("/preferences")
  })

  return (
    <div className="flex min-h-screen flex-col bg-white">
      <AppHead></AppHead>

      {/* Uncomment this to add main 
      <main className="flex flex-1 flex-col items-center justify-center py-8">
        <h1 className="m-0 text-center text-6xl">
          Welcome to{" "}
          <a
            href="https://nextjs.org"
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-600 hover:underline focus:underline active:underline"
          >
            Next.js!
          </a>
        </h1>

        <p className="my-12 text-center text-2xl leading-9">
          Get started by editing{" "}
          <code className="rounded-md bg-gray-200 p-2 font-mono text-xl">
            src/pages/index.tsx
          </code>
        </p>

        <div className="flex max-w-3xl flex-wrap items-center justify-center">
          <Card
            url="https://nextjs.org/docs"
            title="Documentation"
            description="Find in-depth information about Next.js features and API."
          />

          <Card
            url="https://nextjs.org/learn"
            title="Learn"
            description="Learn about Next.js in an interactive course with quizzes!"
          />

          <Card
            url="https://github.com/vercel/next.js/tree/canary/examples"
            title="Examples"
            description="Discover and deploy boilerplate example Next.js projects."
          />

          <CardButton
            onClick={onButtonClick}
            title="Tauri Invoke"
            description={buttonDesc}
          />
        </div>
      </main>
      */}

      {/* Uncomment this to add a footer
      <footer className="flex flex-1 flex-grow-0 items-center justify-center border-t border-gray-200 py-4">
        <div>
          <a
            href="https://tauri.app/"
            target="_blank"
            rel="noopener noreferrer"
            className="flex flex-grow items-center justify-center p-4"
          >
            Powered by{" "}
            <span className="ml-2 h-6">
              <Image
                src="/tauri_logo_light.svg"
                alt="Vercel Logo"
                height={24}
                width={78}
              />
            </span>
          </a>
        </div>
      </footer>
      */}
    </div>
  )
}

export default Home
