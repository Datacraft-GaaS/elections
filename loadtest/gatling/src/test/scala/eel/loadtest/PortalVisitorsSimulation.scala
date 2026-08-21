package eel.loadtest

import io.gatling.core.Predef._
import io.gatling.http.Predef._
import scala.concurrent.duration._

/**
 * Gatling simulation: public results portal at 5M viewers/hour.
 *
 * Data-driven: county IDs cycled from a CSV feeder so every request hits a
 * different county endpoint, exercising cache partitioning across 47 counties.
 *
 * Appendix-A SLA: p95 <= 200ms, p99 <= 500ms @ 5M/hr (= 1389 req/s).
 * Proof endpoint: p99 <= 30s.
 *
 * Compile: sbt "Gatling/test" (requires sbt + Java 17+)
 * Skip: document in CI if sbt not installed; k6/portal-visitors.js covers the same endpoint.
 */
class PortalVisitorsSimulation extends Simulation {

  val baseUrl: String = sys.env.getOrElse("EEL_BASE_URL", "http://localhost:8080")

  // Scale factor from env (default 0.01 = local)
  val scalingFactor: Double = sys.env.getOrElse("EEL_SCALING_FACTOR", "0.01").toDouble

  // Target arrival rate at 1x = 1389 req/s (5M/hr). Scaled for current profile.
  val targetRps: Int = math.max(1, (1389 * scalingFactor).toInt)

  val httpProtocol = http
    .baseUrl(baseUrl)
    .acceptHeader("application/json, text/html")
    .acceptLanguageHeader("en-US,sw-KE")
    .userAgentHeader("EelLoadTest/1.0 (Gatling)")

  // 47 Kenyan counties (cycled)
  val countyFeeder = Iterator.continually((1 to 47).map(i => Map("countyId" -> i))).flatten

  // Station feeder for proof endpoint
  val stationFeeder = Iterator.continually(
    (1 to 47000).map(i => Map("stationCode" -> s"STN-$i"))
  ).flatten

  val resultsScenario = scenario("Portal results visitors")
    .feed(countyFeeder)
    .exec(
      http("GET county results")
        .get("/results/county/#{countyId}")
        .check(status.is(200))
        .check(responseTimeInMillis.lt(200))  // p95 budget
    )
    .pause(1, 5)  // user browse time
    .exec(
      http("GET national summary")
        .get("/results")
        .check(status.is(200))
    )

  val proofScenario = scenario("Proof viewers")
    .feed(stationFeeder)
    .exec(
      http("GET inclusion proof")
        .get("/results/proof/#{stationCode}")
        .check(status.is(200))
        .check(responseTimeInMillis.lt(30000))  // Appendix-A: proofs <= 30s
    )
    .pause(5)

  setUp(
    resultsScenario.inject(
      rampUsersPerSec(0).to(targetRps).during(2.minutes),
      constantUsersPerSec(targetRps).during(8.minutes),
      rampUsersPerSec(targetRps).to(0).during(2.minutes)
    ),
    proofScenario.inject(
      nothingFor(2.minutes),
      constantConcurrentUsers(math.max(1, (10 * scalingFactor).toInt)).during(8.minutes)
    )
  ).protocols(httpProtocol)
    .assertions(
      // Appendix-A SLA
      global.responseTime.percentile(95).lt(200),
      global.responseTime.percentile(99).lt(500),
      global.failedRequests.percent.lt(0.1)
    )
}
