import type { ReactNode } from "react";
import Layout from "@theme/Layout";
import Link from "@docusaurus/Link";
import Translate, { translate } from "@docusaurus/Translate";
import useBaseUrl from "@docusaurus/useBaseUrl";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import styles from "./index.module.css";

const FEATURES = [
  {
    id: "widget",
    image: "widget-controls",
    title: <Translate id="home.widget.title">A tomato on your desktop</Translate>,
    text: (
      <Translate id="home.widget.text">
        A small always-on-top widget with the countdown in the middle and a ring that shows
        progress. Red for work, green for a short break, blue for a long break, amber for a
        countdown. The buttons appear when you hover over it.
      </Translate>
    ),
  },
  {
    id: "cycle",
    image: "settings",
    title: <Translate id="home.cycle.title">Your cycle, your rules</Translate>,
    text: (
      <Translate id="home.cycle.text">
        The classic 25/5/15 profile is the default, with a one-click way back to it. Change the
        durations, the number of sessions before a long break, the daily goal, and whether the
        next phase starts by itself.
      </Translate>
    ),
  },
  {
    id: "alerts",
    image: "settings-alerts",
    title: <Translate id="home.alerts.title">Alerts you choose</Translate>,
    text: (
      <Translate id="home.alerts.text">
        For each event, pick any mix of a system notification, a sound, a full-screen overlay and
        bringing the widget to the front. Use the built-in sounds or your own files.
      </Translate>
    ),
  },
  {
    id: "breaks",
    image: "overlay-break",
    title: <Translate id="home.breaks.title">Breaks with an exercise</Translate>,
    text: (
      <Translate id="home.breaks.text">
        Keep a list of planks, sit-ups or stretches. Each break shows the next one, and a long
        break is split into several short ones, each with its own exercise.
      </Translate>
    ),
  },
  {
    id: "stats",
    image: "statistics",
    title: <Translate id="home.stats.title">Statistics, kept on your computer</Translate>,
    text: (
      <Translate id="home.stats.text">
        Daily, weekly and monthly charts, streaks, completion rate and your best time of day.
        Everything is stored in a local SQLite file. Nothing is sent anywhere.
      </Translate>
    ),
  },
];

/** The images are rendered once per language, French ones in the fr folder. */
function useScreenshot(name: string): string {
  const { i18n } = useDocusaurusContext();
  const folder = i18n.currentLocale === "en" ? "" : `${i18n.currentLocale}/`;
  return useBaseUrl(`/img/screenshots/${folder}${name}.png`);
}

function Hero(): ReactNode {
  const work = useScreenshot("widget-work");
  const shortBreak = useScreenshot("widget-short-break");
  return (
    <header className={styles.hero}>
      <div className={`container ${styles.heroInner}`}>
        <div className={styles.heroText}>
          <h1>Tomatick</h1>
          <p className={styles.tagline}>
            <Translate id="home.tagline">
              A free, open source Pomodoro timer for Windows, macOS and Linux.
            </Translate>
          </p>
          <p>
            <Translate id="home.pure">
              A pure timer: no task list, no account, no cloud. A tomato sits on your desktop and
              gets out of your way.
            </Translate>
          </p>
          <div className={styles.buttons}>
            <Link className="button button--primary button--lg" to="/docs/installation">
              <Translate id="home.install">Install</Translate>
            </Link>
            <Link className="button button--secondary button--lg" to="/docs/user-guide">
              <Translate id="home.guide">Read the user guide</Translate>
            </Link>
          </div>
        </div>
        <div className={styles.heroImages}>
          <img
            className="replaceable-image"
            src={work}
            alt={translate({ id: "home.alt.work", message: "The Tomatick widget during a work phase" })}
            width={200}
            height={200}
          />
          <img
            className="replaceable-image"
            src={shortBreak}
            alt={translate({ id: "home.alt.break", message: "The Tomatick widget during a short break" })}
            width={200}
            height={200}
          />
        </div>
      </div>
    </header>
  );
}

function Feature({ feature, flip }: { feature: (typeof FEATURES)[number]; flip: boolean }): ReactNode {
  return (
    <section className={`${styles.feature} ${flip ? styles.flip : ""}`}>
      <div className={styles.featureText}>
        <h2>{feature.title}</h2>
        <p>{feature.text}</p>
      </div>
      <div className={styles.featureImage}>
        <img
          className="replaceable-image"
          src={useScreenshot(feature.image)}
          alt=""
          loading="lazy"
        />
      </div>
    </section>
  );
}

export default function Home(): ReactNode {
  return (
    <Layout
      title={translate({ id: "home.title", message: "A free Pomodoro timer" })}
      description={translate({
        id: "home.description",
        message:
          "Tomatick is a free, open source Pomodoro timer with a small always-on-top tomato widget, for Windows, macOS and Linux.",
      })}
    >
      <Hero />
      <main className={`container ${styles.features}`}>
        {FEATURES.map((feature, index) => (
          <Feature key={feature.id} feature={feature} flip={index % 2 === 1} />
        ))}
        <p className={styles.note}>
          <Translate id="home.note">
            The images on this site were rendered from the interface with sample data. They are
            not captures of a real desktop and will be replaced.
          </Translate>
        </p>
      </main>
    </Layout>
  );
}
