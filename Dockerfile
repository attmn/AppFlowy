# Step 1: Build the Backend (assuming Rust is used for the backend)
FROM rust:1.60.0 as backend-builder

# Set the working directory
WORKDIR /app

# Copy the backend source code
COPY backend/ .

# Install any necessary dependencies and build the project
RUN apt-get update && apt-get install -y libssl-dev pkg-config && \
    cargo build --release

# Step 2: Build the Frontend
FROM node:16 as frontend-builder

# Set the working directory
WORKDIR /app

# Copy the frontend source code
COPY frontend/package.json frontend/yarn.lock ./

# Install dependencies
RUN yarn install

# Copy the rest of the frontend code and build it
COPY frontend/ .
RUN yarn build

# Step 3: Use a lightweight server image to run the built application
FROM nginx:alpine

# Copy the built backend and frontend
COPY --from=backend-builder /app/target/release/backend /usr/share/nginx/html/backend
COPY --from=frontend-builder /app/build /usr/share/nginx/html/frontend

# Expose the application port
EXPOSE 80

# Start the Nginx server
CMD ["nginx", "-g", "daemon off;"]
