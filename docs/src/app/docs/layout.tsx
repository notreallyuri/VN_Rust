import { PageNav } from "@/components/page-nav";
import { Sidebar } from "@/components/sidebar";
import { SiteHeader } from "@/components/site-header";
import { Toc } from "@/components/toc";

export default function DocsLayout({ children }: LayoutProps<"/docs">) {
  return (
    <>
      <SiteHeader />
      <div className="mx-auto flex w-full max-w-6xl gap-10 px-6">
        <aside className="sticky top-[4.5rem] hidden h-[calc(100vh-6rem)] w-52 shrink-0 overflow-y-auto py-12 lg:block">
          <Sidebar />
        </aside>
        <main className="min-w-0 max-w-2xl flex-1 py-12">
          {children}
          <PageNav />
        </main>
        <aside className="sticky top-[4.5rem] hidden h-[calc(100vh-6rem)] w-48 shrink-0 overflow-y-auto py-12 xl:block">
          <Toc />
        </aside>
      </div>
    </>
  );
}
