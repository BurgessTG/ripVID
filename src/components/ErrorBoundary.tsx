import React, { Component, ErrorInfo, ReactNode } from "react";
import "./ErrorBoundary.css";

interface Props {
    children: ReactNode;
    fallback?: ReactNode;
}

interface State {
    hasError: boolean;
    error: Error | null;
    errorInfo: ErrorInfo | null;
}

/**
 * Error Boundary Component
 * Catches JavaScript errors anywhere in the child component tree and displays a fallback UI
 */
class ErrorBoundary extends Component<Props, State> {
    constructor(props: Props) {
        super(props);
        this.state = {
            hasError: false,
            error: null,
            errorInfo: null,
        };
    }

    static getDerivedStateFromError(error: Error): Partial<State> {
        return { hasError: true, error };
    }

    componentDidCatch(error: Error, errorInfo: ErrorInfo) {
        console.error("ErrorBoundary caught an error:", error, errorInfo);
        this.setState({ error, errorInfo });
    }

    handleReload = () => {
        window.location.reload();
    };

    handleReset = () => {
        this.setState({
            hasError: false,
            error: null,
            errorInfo: null,
        });
    };

    render() {
        if (this.state.hasError) {
            if (this.props.fallback) {
                return this.props.fallback;
            }

            return (
                <div className="error-boundary">
                    <div className="error-boundary-content">
                        <div className="error-icon">⚠️</div>
                        <h1>Oops! Something went wrong</h1>
                        <p className="error-message">
                            The application encountered an unexpected error.
                        </p>

                        {this.state.error && (
                            <details className="error-details">
                                <summary>Error Details</summary>
                                <pre className="error-stack">
                                    {this.state.error.toString()}
                                    {this.state.errorInfo?.componentStack}
                                </pre>
                            </details>
                        )}

                        <div className="error-actions">
                            <button className="error-button primary" onClick={this.handleReload}>
                                Reload App
                            </button>
                            <button className="error-button secondary" onClick={this.handleReset}>
                                Try Again
                            </button>
                            <a
                                href="https://github.com/yourusername/ripvid/issues"
                                target="_blank"
                                rel="noopener noreferrer"
                                className="error-button secondary"
                            >
                                Report Issue
                            </a>
                        </div>

                        <p className="error-hint">
                            If this problem persists, try clearing your browser cache or reinstalling the application.
                        </p>
                    </div>
                </div>
            );
        }

        return this.props.children;
    }
}

export default ErrorBoundary;
