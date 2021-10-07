@extends('next.base')

@push('scripts')
    @vite('resources/lib/index.ts')
@endpush

@section('app')
    <div id="app"></div>
@endsection
