package abis

import io.gatling.core.Predef._
import io.gatling.http.Predef._
import scala.concurrent.duration._
import java.util.UUID

/**
 * Gatling simulation: combined 30-minute ABIS scenario.
 * Runs all four k6 profile equivalents concurrently for 30 minutes.
 * ADR-0018 SLA validation.
 *
 * Run: gatling.sh -s abis.AbisScenarios
 * Env: ABIS_URL (default http://localhost:8080)
 */
class AbisScenarios extends Simulation {

  val baseUrl: String = sys.env.getOrElse("ABIS_URL", "http://localhost:8080")

  val httpProtocol = http
    .baseUrl(baseUrl)
    .acceptHeader("application/json")
    .contentTypeHeader("application/json")
    .header("X-Eel-Caller", "eel-gatling-simulation")

  // --- Feeder: pull from synthetic templates JSONL ---
  val templateFeeder = jsonFile("synthetic-templates/templates.jsonl").circular

  // --- Scenario A: enrolment (1N) ---
  val enrolment = scenario("Enrolment-1N")
    .feed(templateFeeder)
    .exec(
      http("enrol-1n")
        .post("/api/v1/abis/match/1n")
        .header("Idempotency-Key", _ => s"eel-enrol-${UUID.randomUUID()}")
        .body(StringBody(session =>
          s"""{
            |"probe_ref":"bio-store://eel.local/probes/${session("id").as[String]}",
            |"template_blob":"${session("fingerprint_b64").as[String]}",
            |"modality":"fingerprint",
            |"caller_id":"spiffe://eel.local/ns/enrolment/sa/enrolment-svc",
            |"election_id":"00000000-0000-7000-8000-000000000001"
            |}""".stripMargin
        )).asJson
        .check(status.is(202))
        .check(responseTimeInMillis.lte(8000))
    )
    .pause(300.milliseconds, 600.milliseconds)

  // --- Scenario B: 1:N match ---
  val match1N = scenario("Match-1N")
    .feed(templateFeeder)
    .exec(
      http("match-1n")
        .post("/api/v1/abis/match/1n")
        .header("Idempotency-Key", _ => s"eel-1n-${UUID.randomUUID()}")
        .body(StringBody(session =>
          s"""{
            |"probe_ref":"bio-store://eel.local/probes/${session("id").as[String]}",
            |"template_blob":"${session("fingerprint_b64").as[String]}",
            |"modality":"fingerprint",
            |"caller_id":"spiffe://eel.local/ns/abis/sa/abis-api",
            |"max_candidates":20
            |}""".stripMargin
        )).asJson
        .check(status.is(202))
        .check(responseTimeInMillis.lte(8000))
    )

  // --- Scenario C: 1:1 match ---
  val match11 = scenario("Match-11")
    .feed(templateFeeder)
    .exec(
      http("match-11")
        .post("/api/v1/abis/match/11")
        .header("Idempotency-Key", _ => s"eel-11-${UUID.randomUUID()}")
        .body(StringBody(session =>
          s"""{
            |"probe_ref":"bio-store://eel.local/probes/${session("id").as[String]}",
            |"gallery_ref":"bio-store://eel.local/gallery/voter-${session("id").as[String]}",
            |"template_blob":"${session("fingerprint_b64").as[String]}",
            |"modality":"fingerprint",
            |"caller_id":"spiffe://eel.local/ns/bvi/sa/bvi-backend"
            |}""".stripMargin
        )).asJson
        .check(status.is(200))
        .check(responseTimeInMillis.lte(200))
    )

  // --- Scenario D: kit enrolment with attestation header ---
  val kitEnrolment = scenario("Kit-Enrolment")
    .feed(templateFeeder)
    .exec(
      http("kit-enrol")
        .post("/api/v1/abis/match/1n")
        .header("Idempotency-Key", _ => s"eel-kit-${UUID.randomUUID()}")
        .header("X-Eel-Attestation", "dev-attest-placeholder")
        .header("X-Eel-Kit-Serial", session => s"EKLK-${session("id").as[String].take(8).toUpperCase}")
        .body(StringBody(session =>
          s"""{
            |"probe_ref":"bio-store://eel.local/kit-probes/${session("id").as[String]}",
            |"template_blob":"${session("fingerprint_b64").as[String]}",
            |"modality":"fingerprint+iris",
            |"caller_id":"spiffe://eel.local/ns/kit/sa/enrolment-kit",
            |"election_id":"00000000-0000-7000-8000-000000000001"
            |}""".stripMargin
        )).asJson
        .check(status.is(202))
        .check(responseTimeInMillis.lte(3000))
    )

  setUp(
    // ~2.8 enrolments/sec (10k/hr)
    enrolment.inject(
      rampUsersPerSec(0.5) to 2.8 during (5.minutes),
      constantUsersPerSec(2.8) during (20.minutes),
      rampUsersPerSec(2.8) to 0 during (5.minutes)
    ),
    // 100 concurrent 1:N searchers
    match1N.inject(
      rampUsers(100) during (5.minutes),
      constantConcurrentUsers(100) during (20.minutes),
      rampUsers(0) during (5.minutes)
    ),
    // 500 rps 1:1
    match11.inject(
      rampUsersPerSec(50) to 500 during (5.minutes),
      constantUsersPerSec(500) during (20.minutes),
      rampUsersPerSec(500) to 0 during (5.minutes)
    ),
    // 500 concurrent kit enrolments
    kitEnrolment.inject(
      rampUsers(500) during (3.minutes),
      constantConcurrentUsers(500) during (22.minutes),
      rampUsers(0) during (5.minutes)
    )
  ).protocols(httpProtocol)
    .assertions(
      // 1:N
      details("Enrolment-1N" / "enrol-1n").responseTime.percentile3.lt(8000),
      details("Match-1N"     / "match-1n").responseTime.percentile3.lt(8000),
      details("Match-1N"     / "match-1n").responseTime.percentile(95).lt(5000),
      // 1:1
      details("Match-11" / "match-11").responseTime.percentile3.lt(200),
      details("Match-11" / "match-11").responseTime.percentile(95).lt(100),
      // Kit
      details("Kit-Enrolment" / "kit-enrol").responseTime.percentile(95).lt(3000),
      // Overall failure rate
      global.failedRequests.percent.lt(1.0)
    )
}
