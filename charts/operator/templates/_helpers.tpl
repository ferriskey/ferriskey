{{- define "ferriskey.global.name" -}}
{{ (.Values.nameOverride | default .Release.Name) | trunc 43 }}
{{- end -}}

{{- define "ferriskey.operator.name" -}}
{{ include "ferriskey.global.name" . }}-operator
{{- end -}}

{{- define "ferriskey.global.labels" -}}
app.kubernetes.io/instance: {{ include "ferriskey.global.name" . }}
app.kubernetes.io/version: {{ .Chart.Version }}
app.kubernetes.io/part-of: ferriskey
{{- end -}}

{{- define "ferriskey.global.podLabels" -}}
app.kubernetes.io/instance: {{ include "ferriskey.global.name" . }}
app.kubernetes.io/part-of: ferriskey
{{- end -}}


Expand the name of the chart.
*/}}
{{- define "operator.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "operator.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "operator.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "operator.labels" -}}
helm.sh/chart: {{ include "operator.chart" . }}
{{ include "operator.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "operator.selectorLabels" -}}
app.kubernetes.io/name: {{ include "operator.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "operator.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "operator.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{- define "ferriskey.operator.labels" -}}
{{- $labels := merge .Values.common.labels .Values.operator.labels -}}
{{- include "ferriskey.global.labels" . }}
app.kubernetes.io/name: ferriskey-operator
app.kubernetes.io/component: operator
{{- with $labels }}
{{- toYaml . }}
{{- end }}
{{- end -}}


{{- define "ferriskey.operator.podLabels" -}}
{{- $labels := merge .Values.common.podLabels .Values.operator.podLabels -}}
{{- include "ferriskey.global.podLabels" . }}
app.kubernetes.io/name: ferriskey-api
app.kubernetes.io/component: api
{{- end -}}
