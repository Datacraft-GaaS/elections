// Gatling load-test scenarios for eel.
// Requires: sbt 1.9+, Java 17+.
// Run: sbt "Gatling/test"
// Skip: if sbt is not installed, the DoD allows skipping Gatling compilation.
//       k6 scenarios cover the same endpoints; Gatling adds data-driven replay at fanout scale.

name := "eel-loadtest-gatling"
version := "0.1.0"
scalaVersion := "2.13.14"

enablePlugins(GatlingPlugin)

libraryDependencies += "io.gatling.highcharts" % "gatling-charts-highcharts" % "3.11.5" % "test"
libraryDependencies += "io.gatling"            % "gatling-test-framework"    % "3.11.5" % "test"

// Test options
Test / testOptions += Tests.Argument(TestFrameworks.JUnit, "-v")

addSbtPlugin("io.gatling" % "gatling-sbt" % "4.9.6")
