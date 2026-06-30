import { NextResponse } from "next/server";

/** Liveness probe for load balancers and Kubernetes. */
export function GET() {
  return NextResponse.json(
    { status: "ok", service: "neuralscope-web" },
    { status: 200 },
  );
}
