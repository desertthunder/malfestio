import { Footer } from "$components/layout/Footer";
import { A } from "@solidjs/router";
import type { Component, JSX } from "solid-js";
import { For } from "solid-js";
import { Motion } from "solid-motionone";

const features = [{
  title: "Flashcards",
  desc: "Built-in spaced repetition system (SRS) ensuring you review the right material at the right time.",
  icon: <span class="i-bi-card-text text-4xl" />,
}, {
  title: "Linked Notes",
  desc: "Connect concepts with bidirectional links. Build a knowledge graph that grows with your understanding.",
  icon: <span class="i-bi-link-45deg text-4xl" />,
}, {
  title: "Lectures & Articles",
  desc: "Import content directly. Highlight, annotate, and turn key insights into flashcards instantly.",
  icon: <span class="i-bi-book text-4xl" />,
}, {
  title: "Social Learning",
  desc: "Publish your decks, follow curators, and fork existing content to improve it for everyone.",
  icon: <span class="i-bi-people text-4xl" />,
}, {
  title: "Local-First",
  desc: "Your data lives on your device. Offline-first architecture with ATProto for decentralized sync.",
  icon: <span class="i-bi-hdd text-4xl" />,
}, {
  title: "Open Source",
  desc: "Validates knowledge, not proprietary locks. Inspect the code, extend the schema, own the platform.",
  icon: (
    <svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 25 25" class="text-4xl">
      <path
        fill="currentColor"
        d="m 16.208435,23.914069 c -0.06147,-0.02273 -0.147027,-0.03034 -0.190158,-0.01691 -0.197279,0.06145 -1.31068,-0.230493 -1.388819,-0.364153 -0.01956,-0.03344 -0.163274,-0.134049 -0.319377,-0.223561 -0.550395,-0.315603 -1.010951,-0.696643 -1.428383,-1.181771 -0.264598,-0.307509 -0.597257,-0.785384 -0.597257,-0.857979 0,-0.0216 -0.02841,-0.06243 -0.06313,-0.0907 -0.04977,-0.04053 -0.160873,0.0436 -0.52488,0.397463 -0.479803,0.466432 -0.78924,0.689475 -1.355603,0.977118 -0.183693,0.0933 -0.323426,0.179989 -0.310516,0.192658 0.02801,0.02748 -0.7656391,0.270031 -1.209129,0.369517 -0.5378332,0.120647 -1.6341809,0.08626 -1.9721503,-0.06186 C 6.7977157,23.031391 6.56735,22.957551 6.3371134,22.889782 4.9717169,22.487902 3.7511914,21.481518 3.1172396,20.234838 2.6890391,19.392772 2.5582276,18.827446 2.5610489,17.831154 2.5639589,16.802192 2.7366641,16.125844 3.2142117,15.273187 3.3040457,15.112788 3.3713143,14.976533 3.3636956,14.9704 3.3560756,14.9643 3.2459634,14.90305 3.1189994,14.834381 1.7582586,14.098312 0.77760984,12.777439 0.44909837,11.23818 0.33531456,10.705039 0.33670119,9.7067968 0.45195381,9.1778795 0.72259241,7.9359287 1.3827188,6.8888436 2.4297498,6.0407205 2.6856126,5.8334648 3.2975489,5.4910878 3.6885849,5.3364049 L 4.0584319,5.190106 4.2333984,4.860432 C 4.8393906,3.7186139 5.8908314,2.7968028 7.1056396,2.3423025 7.7690673,2.0940921 8.2290216,2.0150935 9.01853,2.0137575 c 0.9625627,-0.00163 1.629181,0.1532762 2.485864,0.5776514 l 0.271744,0.1346134 0.42911,-0.3607688 c 1.082666,-0.9102346 2.185531,-1.3136811 3.578383,-1.3090327 0.916696,0.00306 1.573918,0.1517893 2.356121,0.5331927 1.465948,0.7148 2.54506,2.0625628 2.865177,3.57848 l 0.07653,0.362429 0.515095,0.2556611 c 1.022872,0.5076874 1.756122,1.1690944 2.288361,2.0641468 0.401896,0.6758594 0.537303,1.0442682 0.675505,1.8378683 0.288575,1.6570823 -0.266229,3.3548023 -1.490464,4.5608743 -0.371074,0.36557 -0.840205,0.718265 -1.203442,0.904754 -0.144112,0.07398 -0.271303,0.15826 -0.282647,0.187269 -0.01134,0.02901 0.02121,0.142764 0.07234,0.25279 0.184248,0.396467 0.451371,1.331823 0.619371,2.168779 0.463493,2.30908 -0.754646,4.693707 -2.92278,5.721632 -0.479538,0.227352 -0.717629,0.309322 -1.144194,0.39393 -0.321869,0.06383 -1.850573,0.09139 -2.000174,0.03604 z M 12.25443,18.636956 c 0.739923,-0.24652 1.382521,-0.718922 1.874623,-1.37812 0.0752,-0.100718 0.213883,-0.275851 0.308198,-0.389167 0.09432,-0.113318 0.210136,-0.271056 0.257381,-0.350531 0.416347,-0.700389 0.680936,-1.176102 0.766454,-1.378041 0.05594,-0.132087 0.114653,-0.239607 0.130477,-0.238929 0.01583,6.79e-4 0.08126,0.08531 0.145412,0.188069 0.178029,0.285173 0.614305,0.658998 0.868158,0.743878 0.259802,0.08686 0.656158,0.09598 0.911369,0.02095 0.213812,-0.06285 0.507296,-0.298016 0.645179,-0.516947 0.155165,-0.246374 0.327989,-0.989595 0.327989,-1.410501 0,-1.26718 -0.610975,-3.143405 -1.237774,-3.801045 -0.198483,-0.2082486 -0.208557,-0.2319396 -0.208557,-0.4904655 0,-0.2517771 -0.08774,-0.5704927 -0.258476,-0.938956 C 16.694963,8.50313 16.375697,8.1377479 16.135846,7.9543702 L 15.932296,7.7987471 15.683004,7.9356529 C 15.131767,8.2383821 14.435638,8.1945733 13.943459,7.8261812 L 13.782862,7.7059758 13.686773,7.8908012 C 13.338849,8.5600578 12.487087,8.8811064 11.743178,8.6233891 11.487199,8.5347109 11.358897,8.4505994 11.063189,8.1776138 L 10.69871,7.8411436 10.453484,8.0579255 C 10.318608,8.1771557 10.113778,8.3156283 9.9983037,8.3656417 9.7041488,8.4930449 9.1808299,8.5227884 8.8979004,8.4281886 8.7754792,8.3872574 8.6687415,8.3537661 8.6607053,8.3537661 c -0.03426,0 -0.3092864,0.3066098 -0.3791974,0.42275 -0.041935,0.069664 -0.1040482,0.1266636 -0.1380294,0.1266636 -0.1316419,0 -0.4197402,0.1843928 -0.6257041,0.4004735 -0.1923125,0.2017571 -0.6853701,0.9036038 -0.8926582,1.2706578 -0.042662,0.07554 -0.1803555,0.353687 -0.3059848,0.618091 -0.1256293,0.264406 -0.3270073,0.686768 -0.4475067,0.938581 -0.1204992,0.251816 -0.2469926,0.519654 -0.2810961,0.595199 -0.2592829,0.574347 -0.285919,1.391094 -0.057822,1.77304 0.1690683,0.283105 0.4224039,0.480895 0.7285507,0.568809 0.487122,0.139885 0.9109638,-0.004 1.6013422,-0.543768 l 0.4560939,-0.356568 0.0036,0.172041 c 0.01635,0.781837 0.1831084,1.813183 0.4016641,2.484154 0.1160449,0.356262 0.3781448,0.83968 0.5614081,1.035462 0.2171883,0.232025 0.7140951,0.577268 1.0100284,0.701749 0.121485,0.0511 0.351032,0.110795 0.510105,0.132647 0.396966,0.05452 1.2105,0.02265 1.448934,-0.05679 z" />
    </svg>
  ),
}];

const steps = [{ title: "Import", desc: "Create cards from articles, lectures, or write your own notes." }, {
  title: "Study",
  desc: "Review with spaced repetition — the right card at the right time.",
}, { title: "Share", desc: "Publish to the AT Protocol network and discover community content." }];

const Feature: Component<{ title: string; desc: string; icon: JSX.Element }> = (props) => (
  <div class="border border-neutral-800 p-6 hover:border-blue-600 transition-colors group h-full bg-neutral-900/50 backdrop-blur-sm">
    <div class="w-10 h-10 mb-4 text-blue-500 group-hover:text-blue-400 transition-colors">{props.icon}</div>
    <h3 class="text-xl text-white mb-2 group-hover:text-blue-400 transition-colors">{props.title}</h3>
    <p class="text-neutral-400 font-light leading-relaxed">{props.desc}</p>
  </div>
);

const FloatingCard: Component<{ position: string; delay: number; children: JSX.Element }> = (props) => (
  <Motion.div
    initial={{ opacity: 0, y: 20 }}
    animate={{ opacity: 1, y: 0 }}
    transition={{ duration: 0.8, delay: props.delay, easing: "ease-out" }}
    class={`absolute ${props.position}`}>
    <Motion.div
      animate={{ y: [0, -8, 0] }}
      transition={{ duration: 4, repeat: Infinity, easing: "ease-in-out", delay: props.delay }}>
      {props.children}
    </Motion.div>
  </Motion.div>
);

const StepCard: Component<{ step: number; title: string; desc: string }> = (props) => (
  <div class="flex flex-col items-center text-center">
    <div class="w-12 h-12 rounded-full bg-blue-600 text-white flex items-center justify-center text-lg font-semibold mb-4">
      {props.step}
    </div>
    <h3 class="text-xl font-medium text-white mb-2">{props.title}</h3>
    <p class="text-neutral-400 font-light leading-relaxed">{props.desc}</p>
  </div>
);

const Landing: Component = () => (
  <div class="min-h-screen bg-black text-white font-sans selection:bg-blue-500/30">
    <header class="border-b border-neutral-900 sticky top-0 bg-black/80 backdrop-blur-md z-50">
      <div class="max-w-7xl mx-auto px-6 h-16 flex items-center justify-between">
        <A href="/" class="font-bold tracking-tight text-xl hover:text-blue-400 transition-colors">Malfestio</A>
        <div class="flex items-center gap-6">
          <A href="/about" class="text-sm font-medium text-neutral-400 hover:text-white transition-colors">About</A>
          <A href="/login" class="text-sm font-medium text-neutral-400 hover:text-white transition-colors">Log in</A>
        </div>
      </div>
    </header>
    <main>
      <section class="relative overflow-hidden border-b border-neutral-900">
        <div class="absolute inset-0 grid-pattern" />
        <div class="absolute inset-0 bg-linear-to-b from-transparent via-black/30 to-black" />
        <div class="absolute inset-0 pointer-events-none hidden md:block">
          <FloatingCard position="top-8 right-[3%]" delay={0.1}>
            <div class="bg-blue-600/20 border border-blue-500/30 rounded-lg p-3 w-48 shadow-2xl backdrop-blur-sm">
              <div class="text-xs text-blue-400 mb-1 uppercase tracking-wide">Note</div>
              <div class="text-sm text-blue-100">Spaced repetition optimizes long-term memory retention</div>
            </div>
          </FloatingCard>
          <FloatingCard position="top-4 right-[28%]" delay={0.3}>
            <div class="bg-purple-600/20 border border-purple-500/30 rounded-lg p-3 w-44 shadow-2xl backdrop-blur-sm">
              <div class="text-xs text-purple-400 mb-1 uppercase tracking-wide">Note</div>
              <div class="text-sm text-purple-100">AT Protocol enables decentralized data ownership</div>
            </div>
          </FloatingCard>
          <FloatingCard position="top-40 right-[2%]" delay={0.5}>
            <div class="bg-orange-600/20 border border-orange-500/30 rounded-lg p-3 w-40 shadow-2xl backdrop-blur-sm">
              <div class="text-xs text-orange-400 mb-1 uppercase tracking-wide">Note</div>
              <div class="text-sm text-orange-100">Built with Rust for performance & safety</div>
            </div>
          </FloatingCard>
          <FloatingCard position="top-44 right-[22%]" delay={0.7}>
            <div class="bg-cyan-600/20 border border-cyan-500/30 rounded-lg p-3 w-44 shadow-2xl backdrop-blur-sm">
              <div class="text-xs text-cyan-400 mb-1 uppercase tracking-wide">Note</div>
              <div class="text-sm text-cyan-100">SolidJS for reactive, fine-grained UI updates</div>
            </div>
          </FloatingCard>
          <FloatingCard position="bottom-40 right-[18%]" delay={0.4}>
            <div class="bg-neutral-800/90 border border-neutral-700 rounded-lg p-4 w-60 shadow-2xl backdrop-blur-sm">
              <div class="text-xs text-neutral-500 mb-1 uppercase tracking-wide">Front</div>
              <div class="text-sm text-white leading-relaxed">What is spaced repetition?</div>
            </div>
          </FloatingCard>
          <FloatingCard position="bottom-8 right-[4%]" delay={0.6}>
            <div class="bg-neutral-800/90 border border-neutral-700 rounded-lg p-4 w-64 shadow-2xl backdrop-blur-sm">
              <div class="text-xs text-green-400 mb-1 uppercase tracking-wide">✓ Back</div>
              <div class="text-sm text-white leading-relaxed">
                A learning technique that schedules reviews at optimal intervals
              </div>
            </div>
          </FloatingCard>
        </div>
        <div class="max-w-7xl mx-auto px-6 py-24 md:py-32 relative z-10">
          <div class="max-w-3xl">
            <Motion.h1
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6 }}
              class="text-7xl md:text-8xl font-medium tracking-tight mb-8 leading-[1.1]">
              Learning on <br />
              <h1 class="text-neutral-500">the AT Protocol.</h1>
            </Motion.h1>
            <Motion.p
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.1 }}
              class="text-xl text-neutral-400 font-light mb-12 max-w-2xl leading-relaxed">
              Master complex topics with spaced repetition, linked notes, and active recall. Share your decks, notes,
              and discoveries with the community.
            </Motion.p>
            <Motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.2 }}
              class="flex gap-4">
              <A
                href="/login"
                class="bg-blue-600 hover:bg-blue-700 text-white px-8 py-4 font-medium text-lg transition-colors inline-flex items-center gap-2">
                Get Started
                <span class="text-xl">→</span>
              </A>
            </Motion.div>
          </div>
        </div>
      </section>
      <section class="max-w-7xl mx-auto px-6 py-24">
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
          <For each={features}>{(f) => <Feature title={f.title} desc={f.desc} icon={f.icon} />}</For>
        </div>
      </section>
      <section class="border-t border-neutral-900 py-24 relative">
        <div class="absolute inset-0 grid-pattern" />
        <div class="absolute inset-0 bg-linear-to-t from-transparent via-black/30 to-black" />
        <div class="max-w-5xl mx-auto px-6 relative z-10">
          <h2 class="text-3xl md:text-4xl font-light text-center mb-16">How it works</h2>
          <div class="flex flex-col md:flex-row items-center justify-center gap-8 md:gap-4">
            <For each={steps}>
              {(s, i) => (
                <>
                  <StepCard step={i() + 1} title={s.title} desc={s.desc} />
                  {i() < steps.length - 1 && <div class="hidden md:block text-neutral-500 text-3xl">→</div>}
                </>
              )}
            </For>
          </div>
        </div>
      </section>
    </main>
    <Footer />
  </div>
);

export default Landing;
