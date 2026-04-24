<script>
  let email = '';
  let password = '';
  let loading = false;
  let error = '';
  let success = false;

  const API_URL = import.meta.env.PUBLIC_API_URL || 'http://localhost:3000';

  async function handleLogin() {
    if (!email || !password) {
      error = 'Please enter email and password';
      return;
    }

    loading = true;
    error = '';

    try {
      const res = await fetch(`${API_URL}/api/v1/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password })
      });

      const data = await res.json();
      
      if (res.ok) {
        localStorage.setItem('token', data.access_token);
        success = true;
        window.location.href = '/dashboard';
      } else {
        error = data.error || 'Login failed';
      }
    } catch (e) {
      error = 'Connection error. Is the server running?';
    } finally {
      loading = false;
    }
  }
</script>

<main>
  <div class="login-container">
    <h1>Secure School ERP</h1>
    <p class="subtitle">Sign in to your account</p>

    {#if error}
      <div class="error">{error}</div>
    {/if}

    <form on:submit|preventDefault={handleLogin}>
      <div class="form-group">
        <label for="email">Email</label>
        <input 
          type="email" 
          id="email" 
          bind:value={email}
          placeholder="Enter your email"
          disabled={loading}
        />
      </div>

      <div class="form-group">
        <label for="password">Password</label>
        <input 
          type="password" 
          id="password" 
          bind:value={password}
          placeholder="Enter your password"
          disabled={loading}
        />
      </div>

      <button type="submit" disabled={loading}>
        {loading ? 'Signing in...' : 'Sign In'}
      </button>
    </form>
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: #f5f5f5;
  }

  main {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .login-container {
    background: white;
    padding: 2rem;
    border-radius: 8px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    width: 100%;
    max-width: 400px;
  }

  h1 {
    margin: 0 0 0.5rem;
    color: #333;
    font-size: 1.5rem;
  }

  .subtitle {
    color: #666;
    margin-bottom: 1.5rem;
  }

  .error {
    background: #fee;
    color: #c00;
    padding: 0.75rem;
    border-radius: 4px;
    margin-bottom: 1rem;
  }

  .form-group {
    margin-bottom: 1rem;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    color: #333;
    font-weight: 500;
  }

  input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 1rem;
    box-sizing: border-box;
  }

  input:focus {
    outline: none;
    border-color: #0066cc;
  }

  button {
    width: 100%;
    padding: 0.75rem;
    background: #0066cc;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 1rem;
    cursor: pointer;
  }

  button:disabled {
    background: #ccc;
    cursor: not-allowed;
  }

  button:hover:not(:disabled) {
    background: #0052a3;
  }
</style>