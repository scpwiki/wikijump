@extends('next.base')

@push('scripts')
    @vite('resources/lib/index.ts')
@endpush

@section('content')
    <div id="app"></div>
@endsection
