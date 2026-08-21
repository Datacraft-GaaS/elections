package eel.loadtest

import io.gatling.core.Predef._
import io.gatling.http.Predef._
import scala.concurrent.duration._

/**
 * Gatling simulation: public verifier / M13 at 5M-viewers scale.
 *
 * Two scenarios:
 *   1. REST poll: voter checks their participation token via GET /verify/{voter_id}.
 *   2. SSE stream: live-result viewer maintains a Server-Sent Events connection.
 *
 * Appendix-A SLA: 10M concurrent, p99 < 500ms.
 * Data-driven: voter IDs drawn from a 22M-record population via modular iteration.
 *
 * Compile: sbt "Gatling/test"
 * Skip: document in CI if sbt not installed; k6/verifier-mobile.js covers the same endpoints.
 */
class VerifierMobileSimulation extends Simulation {

  val baseUrl: String = sys.env.getOrElse("EEL_BASE_URL", "http://localhost:8080")

  val scalingFactor: Double = sys.env.getOrElse("EEL_SCALING_FACTOR", "0.01").toDouble

  // 10M concurrent at 1x; scale down for local/staging
  val targetVus: Int = math.max(5, (10_000_000 * scalingFactor).toInt)

  val httpProtocol = http
    .baseUrl(baseUrl)
    .acceptHeader("application/json, text/event-stream")
    .userAgentHeader("EelLoadTest/1.0 (Gatling)")

  // Voter ID feeder: cycles through 22M IDs
  val voterFeeder = Iterator.continually {
    val id = math.abs(scala.util.Random.nextInt(22_000_000))
    Map("voterId" -> s"V$id")
  }

  val restVerifierScenario = scenario("REST participation-token verifier")
    .feed(voterFeeder)
    .exec(
      http("GET verify voter")
        .get("/verify/#{voterId}")
        .check(status.in(200, 404))  // 404 = not on register (valid)
        .check(responseTimeInMillis.lt(500))
    )
    .pause(200.milliseconds, 2.seconds)

  val sseViewerScenario = scenario("SSE live-results viewer")
    .feed(voterFeeder)
    .exec(
      http("SSE connect")
        .get("/verifier/stream")
        .header("Accept", "text/event-stream")
        .check(status.is(200))
        .check(responseTimeInMillis.lt(500))
    )
    .pause(5.seconds, 15.seconds)

  setUp(
    restVerifierScenario.inject(
      rampConcurrentUsers(0).to(math.floor(targetVus * 0.7).toInt).during(2.minutes),
      constantConcurrentUsers(math.floor(targetVus * 0.7).toInt).during(6.minutes),
      rampConcurrentUsers(math.floor(targetVus * 0.7).toInt).to(0).during(2.minutes)
    ),
    sseViewerScenario.inject(
      nothingFor(1.minute),
      constantConcurrentUsers(math.floor(targetVus * 0.3).toInt).during(8.minutes)
    )
  ).protocols(httpProtocol)
    .assertions(
      // Appendix-A SLA: p99 < 500ms
      global.responseTime.percentile(99).lt(500),
      global.failedRequests.percent.lt(1.0)
    )
}
